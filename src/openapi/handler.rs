#![cfg_attr(not(feature = "auth"), allow(unused_imports))]
use super::SECURITY_NAME;
use base64::prelude::*;
use futures_util::{future, future::FutureExt};
use gotham::{
	anyhow,
	handler::{Handler, HandlerError, HandlerFuture, NewHandler},
	helpers::http::response::{create_empty_response, create_response},
	hyper::{
		header::{
			HeaderMap, HeaderValue, CACHE_CONTROL, CONTENT_SECURITY_POLICY, ETAG, IF_NONE_MATCH,
			REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS
		},
		Body, Response, StatusCode
	},
	mime::{APPLICATION_JSON, TEXT_HTML_UTF_8, TEXT_PLAIN_UTF_8},
	state::State
};
use gotham_restful_redoc::Redoc;
use openapi_type::{
	indexmap::IndexMap,
	openapiv3::{APIKeyLocation, OpenAPI, ReferenceOr, SecurityScheme}
};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::{panic::RefUnwindSafe, pin::Pin, sync::Arc};

#[cfg(feature = "auth")]
fn get_security(state: &State) -> IndexMap<String, ReferenceOr<SecurityScheme>> {
	use crate::AuthSource;
	use gotham::state::FromState;

	let source = match AuthSource::try_borrow_from(state) {
		Some(source) => source,
		None => return Default::default()
	};

	let security_scheme = match source {
		AuthSource::Cookie(name) => SecurityScheme::APIKey {
			location: APIKeyLocation::Cookie,
			name: name.to_string(),
			description: None,
			extensions: Default::default()
		},
		AuthSource::Header(name) => SecurityScheme::APIKey {
			location: APIKeyLocation::Header,
			name: name.to_string(),
			description: None,
			extensions: Default::default()
		},
		AuthSource::AuthorizationHeader => SecurityScheme::HTTP {
			scheme: "bearer".to_owned(),
			bearer_format: Some("JWT".to_owned()),
			description: None,
			extensions: Default::default()
		}
	};

	let mut security_schemes: IndexMap<String, ReferenceOr<SecurityScheme>> = Default::default();
	security_schemes.insert(SECURITY_NAME.to_owned(), ReferenceOr::Item(security_scheme));

	security_schemes
}

#[cfg(not(feature = "auth"))]
fn get_security(_state: &State) -> IndexMap<String, ReferenceOr<SecurityScheme>> {
	Default::default()
}

fn openapi_string(
	state: &State,
	openapi: &Arc<RwLock<OpenAPI>>
) -> Result<String, serde_json::Error> {
	let openapi = openapi.read();

	let mut openapi = openapi.clone();
	let security_schemes = get_security(state);
	let mut components = openapi.components.unwrap_or_default();
	components.security_schemes = security_schemes;
	openapi.components = Some(components);

	serde_json::to_string(&openapi)
}

fn create_openapi_response(state: &State, openapi: &Arc<RwLock<OpenAPI>>) -> Response<Body> {
	match openapi_string(state, openapi) {
		Ok(body) => {
			let mut res = create_response(state, StatusCode::OK, APPLICATION_JSON, body);
			let headers = res.headers_mut();
			headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
			res
		},
		Err(e) => {
			error!("Unable to handle OpenAPI request due to error: {e}");
			create_response(
				state,
				StatusCode::INTERNAL_SERVER_ERROR,
				TEXT_PLAIN_UTF_8,
				""
			)
		}
	}
}

#[derive(Clone)]
pub(crate) struct OpenapiSpecHandler {
	openapi: Arc<RwLock<OpenAPI>>
}

// safety: the handler only ever aquires a read lock, so this usage of
// RwLock is, in fact, unwind safe
impl RefUnwindSafe for OpenapiSpecHandler {}

impl OpenapiSpecHandler {
	pub(crate) fn new(openapi: Arc<RwLock<OpenAPI>>) -> Self {
		Self { openapi }
	}
}

impl NewHandler for OpenapiSpecHandler {
	type Instance = Self;

	fn new_handler(&self) -> anyhow::Result<Self> {
		Ok(self.clone())
	}
}

impl Handler for OpenapiSpecHandler {
	fn handle(self, mut state: State) -> Pin<Box<HandlerFuture>> {
		let res = create_openapi_response(&mut state, &self.openapi);
		future::ok((state, res)).boxed()
	}
}

#[derive(Clone)]
pub(crate) struct OpenapiDocHandler {
	openapi: Arc<RwLock<OpenAPI>>
}

// safety: the handler only ever aquires a read lock, so this usage of
// RwLock is, in fact, unwind safe
impl RefUnwindSafe for OpenapiDocHandler {}

impl OpenapiDocHandler {
	pub(crate) fn new(openapi: Arc<RwLock<OpenAPI>>) -> Self {
		Self { openapi }
	}
}

impl NewHandler for OpenapiDocHandler {
	type Instance = Self;

	fn new_handler(&self) -> anyhow::Result<Self> {
		Ok(self.clone())
	}
}

fn redoc_handler(
	state: &State,
	openapi: &Arc<RwLock<OpenAPI>>
) -> Result<Response<Body>, HandlerError> {
	let spec = openapi_string(state, openapi)?;
	let Redoc { html, script_hash } = gotham_restful_redoc::html(spec);

	let mut etag = Sha256::new();
	etag.update(&html);
	let etag = format!("\"{}\"", BASE64_STANDARD.encode(etag.finalize()));

	if state
		.borrow::<HeaderMap>()
		.get(IF_NONE_MATCH)
		.map_or(false, |header| header.as_bytes() == etag.as_bytes())
	{
		let res = create_empty_response(state, StatusCode::NOT_MODIFIED);
		return Ok(res);
	}

	let mut res = create_response(state, StatusCode::OK, TEXT_HTML_UTF_8, html);
	let headers = res.headers_mut();
	headers.insert(
		CACHE_CONTROL,
		HeaderValue::from_static("public,max-age=2592000")
	);
	headers.insert(
		CONTENT_SECURITY_POLICY,
		format!(
			"default-src 'none';base-uri 'none';script-src 'unsafe-inline' https://cdn.jsdelivr.net 'sha256-{script_hash}' 'strict-dynamic';style-src 'unsafe-inline' https://fonts.googleapis.com;font-src https://fonts.gstatic.com;connect-src 'self';img-src blob: data:",
		).parse().unwrap()
	);
	headers.insert(ETAG, etag.parse().unwrap());
	headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
	headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
	Ok(res)
}

impl Handler for OpenapiDocHandler {
	fn handle(self, state: State) -> Pin<Box<HandlerFuture>> {
		match redoc_handler(&state, &self.openapi) {
			Ok(res) => future::ok((state, res)).boxed(),
			Err(err) => future::err((state, err)).boxed()
		}
	}
}
