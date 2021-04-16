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
				headers.insert(
					CONTENT_SECURITY_POLICY,
					format!(
						"default-src 'none';base-uri 'none';script-src 'unsafe-inline' https://cdn.jsdelivr.net 'sha256-{}' 'strict-dynamic';style-src 'unsafe-inline' https://fonts.googleapis.com;font-src https://fonts.gstatic.com;connect-src 'self';img-src data:",
						SWAGGER_UI_SCRIPT_HASH.as_str()
					).parse().unwrap()
				);
				headers.insert(ETAG, SWAGGER_UI_HTML_ETAG.parse().unwrap());
				headers.insert(REFERRER_POLICY, HeaderValue::from_static("strict-origin-when-cross-origin"));
				headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
				future::ok((state, res)).boxed()
			}
		}
	}
}

const SWAGGER_UI_HTML: &str = concat!(
	r#"<!DOCTYPE HTML><html><head><meta charset="UTF-8"/>"#,
	r#"<link rel="stylesheet" href="https://fonts.googleapis.com/css?family=Open+Sans:300,400,700|Source+Code+Pro:300,400,700&display=swap"/>"#,
	r#"</head><body style="margin:0"><div id="redoc"></div><script>"#,
	include_str!("script.js"),
	r#"</script></body></html>"#
);
static SWAGGER_UI_HTML_ETAG: Lazy<String> = Lazy::new(|| {
	let mut hash = Sha256::new();
	hash.update(SWAGGER_UI_HTML);
	let hash = hash.finalize();
	format!(r#""{}""#, base64::encode(hash))
});

const SWAGGER_UI_SCRIPT: &str = include_str!("script.js");
static SWAGGER_UI_SCRIPT_HASH: Lazy<String> = Lazy::new(|| {
	let mut hash = Sha256::new();
	hash.update(SWAGGER_UI_SCRIPT);
	let hash = hash.finalize();
	base64::encode(hash)
});
