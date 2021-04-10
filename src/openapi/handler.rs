#![cfg_attr(not(feature = "auth"), allow(unused_imports))]
use super::SECURITY_NAME;
use futures_util::{future, future::FutureExt};
use gotham::{
	anyhow,
	handler::{Handler, HandlerFuture, NewHandler},
	helpers::http::response::{create_empty_response, create_response},
	hyper::{
		header::{
			HeaderMap, HeaderValue, CACHE_CONTROL, CONTENT_SECURITY_POLICY, ETAG, IF_NONE_MATCH, REFERRER_POLICY,
			X_CONTENT_TYPE_OPTIONS
		},
		Body, Response, StatusCode, Uri
	},
	state::State
};
use indexmap::IndexMap;
use mime::{APPLICATION_JSON, TEXT_HTML, TEXT_PLAIN};
use once_cell::sync::Lazy;
use openapiv3::{APIKeyLocation, OpenAPI, ReferenceOr, SecurityScheme};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::{panic::RefUnwindSafe, pin::Pin, sync::Arc};

#[cfg(feature = "auth")]
fn get_security(state: &mut State) -> IndexMap<String, ReferenceOr<SecurityScheme>> {
	use crate::AuthSource;
	use gotham::state::FromState;

	let source = match AuthSource::try_borrow_from(state) {
		Some(source) => source,
		None => return Default::default()
	};

	let security_scheme = match source {
		AuthSource::Cookie(name) => SecurityScheme::APIKey {
			location: APIKeyLocation::Cookie,
			name: name.to_string()
		},
		AuthSource::Header(name) => SecurityScheme::APIKey {
			location: APIKeyLocation::Header,
			name: name.to_string()
		},
		AuthSource::AuthorizationHeader => SecurityScheme::HTTP {
			scheme: "bearer".to_owned(),
			bearer_format: Some("JWT".to_owned())
		}
	};

	let mut security_schemes: IndexMap<String, ReferenceOr<SecurityScheme>> = Default::default();
	security_schemes.insert(SECURITY_NAME.to_owned(), ReferenceOr::Item(security_scheme));

	security_schemes
}

#[cfg(not(feature = "auth"))]
fn get_security(_state: &mut State) -> IndexMap<String, ReferenceOr<SecurityScheme>> {
	Default::default()
}

fn create_openapi_response(state: &mut State, openapi: &Arc<RwLock<OpenAPI>>) -> Response<Body> {
	let openapi = openapi.read();

	let mut openapi = openapi.clone();
	let security_schemes = get_security(state);
	let mut components = openapi.components.unwrap_or_default();
	components.security_schemes = security_schemes;
	openapi.components = Some(components);

	match serde_json::to_string(&openapi) {
		Ok(body) => {
			let mut res = create_response(&state, StatusCode::OK, APPLICATION_JSON, body);
			let headers = res.headers_mut();
			headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
			res
		},
		Err(e) => {
			error!("Unable to handle OpenAPI request due to error: {}", e);
			create_response(&state, StatusCode::INTERNAL_SERVER_ERROR, TEXT_PLAIN, "")
		}
	}
}

#[derive(Clone)]
pub(crate) struct OpenapiHandler {
	openapi: Arc<RwLock<OpenAPI>>
}

// safety: the handler only ever aquires a read lock, so this usage of
// RwLock is, in fact, unwind safe
impl RefUnwindSafe for OpenapiHandler {}

impl OpenapiHandler {
	pub(crate) fn new(openapi: Arc<RwLock<OpenAPI>>) -> Self {
		Self { openapi }
	}
}

impl NewHandler for OpenapiHandler {
	type Instance = Self;

	fn new_handler(&self) -> anyhow::Result<Self> {
		Ok(self.clone())
	}
}

impl Handler for OpenapiHandler {
	fn handle(self, mut state: State) -> Pin<Box<HandlerFuture>> {
		let res = create_openapi_response(&mut state, &self.openapi);
		future::ok((state, res)).boxed()
	}
}

#[derive(Clone)]
pub(crate) struct SwaggerUiHandler {
	openapi: Arc<RwLock<OpenAPI>>
}

// safety: the handler only ever aquires a read lock, so this usage of
// RwLock is, in fact, unwind safe
impl RefUnwindSafe for SwaggerUiHandler {}

impl SwaggerUiHandler {
	pub(crate) fn new(openapi: Arc<RwLock<OpenAPI>>) -> Self {
		Self { openapi }
	}
}

impl NewHandler for SwaggerUiHandler {
	type Instance = Self;

	fn new_handler(&self) -> anyhow::Result<Self> {
		Ok(self.clone())
	}
}

impl Handler for SwaggerUiHandler {
	fn handle(self, mut state: State) -> Pin<Box<HandlerFuture>> {
		let uri: &Uri = state.borrow();
		let query = uri.query();
		match query {
			// TODO this is hacky
			Some(q) if q.contains("spec") => {
				let res = create_openapi_response(&mut state, &self.openapi);
				future::ok((state, res)).boxed()
			},
			_ => {
				{
					let headers: &HeaderMap = state.borrow();
					if headers
						.get(IF_NONE_MATCH)
						.map_or(false, |etag| etag.as_bytes() == SWAGGER_UI_HTML_ETAG.as_bytes())
					{
						let res = create_empty_response(&state, StatusCode::NOT_MODIFIED);
						return future::ok((state, res)).boxed();
					}
				}

				let mut res = create_response(&state, StatusCode::OK, TEXT_HTML, SWAGGER_UI_HTML.as_bytes());
				let headers = res.headers_mut();
				headers.insert(CACHE_CONTROL, HeaderValue::from_static("public,max-age=2592000"));
				headers.insert(CONTENT_SECURITY_POLICY, format!("default-src 'none'; script-src 'unsafe-inline' 'sha256-{}' 'strict-dynamic'; style-src 'unsafe-inline' https://cdnjs.cloudflare.com; connect-src 'self'; img-src data:;", SWAGGER_UI_SCRIPT_HASH.as_str()).parse().unwrap());
				headers.insert(ETAG, SWAGGER_UI_HTML_ETAG.parse().unwrap());
				headers.insert(REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
				headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
				future::ok((state, res)).boxed()
			}
		}
	}
}

// inspired by https://github.com/swagger-api/swagger-ui/blob/master/dist/index.html
const SWAGGER_UI_HTML: Lazy<&'static String> = Lazy::new(|| {
	let template = indoc::indoc! {
		r#"
		<!DOCTYPE HTML>
		<html lang="en">
		<head>
			<meta charset="UTF-8"/>
			<link rel="stylesheet"
				href="https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/3.46.0/swagger-ui.css"
				integrity="sha512-bnx7V/XrEk9agZpJrkTelwhjx/r53sx2pFAVIRGPt/2TkunsGYiXs0RetrU22ttk74IHNTY2atj77/NsKAXo1w=="
				crossorigin="anonymous" />
			<style>
				html {
					box-sizing: border-box;
					overflow-y: scroll;
				}
				*, *::before, *::after {
					box-sizing: inherit;
				}
				body {
					margin: 0;
					background: #fafafa;
				}
			</style>
		</head>
		<body>
			<div id="swagger-ui"></div>
			<script>{{script}}</script>
		</body>
		</html>
		"#
	};
	Box::leak(Box::new(template.replace("{{script}}", SWAGGER_UI_SCRIPT)))
});
static SWAGGER_UI_HTML_ETAG: Lazy<String> = Lazy::new(|| {
	let mut hash = Sha256::new();
	hash.update(SWAGGER_UI_HTML.as_bytes());
	let hash = hash.finalize();
	let hash = base64::encode(hash);
	format!("\"{}\"", hash)
});
const SWAGGER_UI_SCRIPT: &str = r#"
let s0rdy = false;
let s1rdy = false;

window.onload = function() {
	const cb = function() {
		if (!s0rdy || !s1rdy)
			return;
		const ui = SwaggerUIBundle({
			url: window.location.origin + window.location.pathname + '?spec',
			dom_id: '#swagger-ui',
			deepLinking: true,
			presets: [
				SwaggerUIBundle.presets.apis,
				SwaggerUIStandalonePreset
			],
			plugins: [
				SwaggerUIBundle.plugins.DownloadUrl
			],
			layout: 'StandaloneLayout'
		});
		window.ui = ui;
	};
	
	const s0 = document.createElement('script');
	s0.setAttribute('src', 'https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/3.46.0/swagger-ui-bundle.js');
	s0.setAttribute('integrity', 'sha512-2G8MoOYwQseZnuEIsdM/qDr4imwopde6P0X4Nz561D+CMq+ouQ6Dn1WflY8Cj7R5k563YY9fl2A4JMX45CPZCw==');
	s0.setAttribute('crossorigin', 'anonymous');
	s0.onload = function() {
		s0rdy = true;
		cb();
	};
	document.head.appendChild(s0);
	
	const s1 = document.createElement('script');
	s1.setAttribute('src', 'https://cdnjs.cloudflare.com/ajax/libs/swagger-ui/3.46.0/swagger-ui-standalone-preset.js');
	s1.setAttribute('integrity', 'sha512-UO6AQ8HFTSdUk3aEeGjzApwWZ3E6pEWt91jlw8sOI2furXXdCg3tuXvBW5YqFqwvWhF4x/68R9P0xTI3i+PYOg==');
	s1.setAttribute('crossorigin', 'anonymous');
	s1.onload = function() {
		s1rdy = true;
		cb();
	};
	document.head.appendChild(s1);
};
"#;
static SWAGGER_UI_SCRIPT_HASH: Lazy<String> = Lazy::new(|| {
	let mut hash = Sha256::new();
	hash.update(SWAGGER_UI_SCRIPT);
	let hash = hash.finalize();
	base64::encode(hash)
});
