#![cfg_attr(not(feature = "auth"), allow(unused_imports))]
use super::SECURITY_NAME;
use either::Either;
use futures_util::{future, future::FutureExt};
use gotham::{
	anyhow,
	handler::{Handler, HandlerError, HandlerFuture, NewHandler},
	helpers::http::response::{create_empty_response, create_response},
	hyper::{
		header::{
			HeaderMap, HeaderValue, CACHE_CONTROL, CONTENT_SECURITY_POLICY, ETAG, IF_NONE_MATCH, REFERRER_POLICY,
			X_CONTENT_TYPE_OPTIONS
		},
		Body, Response, StatusCode
	},
	state::State
};
use mime::{APPLICATION_JSON, TEXT_HTML_UTF_8, TEXT_PLAIN_UTF_8};
use openapi_type::{
	indexmap::IndexMap,
	openapi::{APIKeyLocation, OpenAPI, ReferenceOr, SecurityScheme}
};
use parking_lot::RwLock;
use sha2::{Digest, Sha256};
use std::{io::Write, iter, panic::RefUnwindSafe, pin::Pin, sync::Arc};

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
			description: None
		},
		AuthSource::Header(name) => SecurityScheme::APIKey {
			location: APIKeyLocation::Header,
			name: name.to_string(),
			description: None
		},
		AuthSource::AuthorizationHeader => SecurityScheme::HTTP {
			scheme: "bearer".to_owned(),
			bearer_format: Some("JWT".to_owned()),
			description: None
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

fn openapi_string(state: &State, openapi: &Arc<RwLock<OpenAPI>>) -> Result<String, serde_json::Error> {
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
			error!("Unable to handle OpenAPI request due to error: {}", e);
			create_response(state, StatusCode::INTERNAL_SERVER_ERROR, TEXT_PLAIN_UTF_8, "")
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

fn redoc_handler(state: &State, openapi: &Arc<RwLock<OpenAPI>>) -> Result<Response<Body>, HandlerError> {
	let spec = openapi_string(state, openapi)?;
	let encoded_spec = spec
		.chars()
		.flat_map(|c| match c {
			'&' => Either::Left("&amp;".chars()),
			'<' => Either::Left("&lt;".chars()),
			'>' => Either::Left("&gt;".chars()),
			c => Either::Right(iter::once(c))
		})
		.collect::<String>();

	let script = include_str!("script.min.js");
	let mut script_hash = Sha256::new();
	script_hash.update(&script);
	let script_hash = base64::encode(script_hash.finalize());

	let mut buf = Vec::<u8>::new();
	write!(
		buf,
		r#"<!DOCTYPE HTML><html><head><meta charset="utf-8"/><meta name="viewport" content="width=device-width,initial-scale=1"/></head>"#
	)?;
	write!(
		buf,
		r#"<body style="margin:0"><div id="spec" style="display:none">{}</div><div id="redoc"></div><script>{}</script></body></html>"#,
		encoded_spec, script
	)?;

	let mut etag = Sha256::new();
	etag.update(&buf);
	let etag = format!("\"{}\"", base64::encode(etag.finalize()));

	if state
		.borrow::<HeaderMap>()
		.get(IF_NONE_MATCH)
		.map_or(false, |header| header.as_bytes() == etag.as_bytes())
	{
		let res = create_empty_response(&state, StatusCode::NOT_MODIFIED);
		return Ok(res);
	}

	let mut res = create_response(state, StatusCode::OK, TEXT_HTML_UTF_8, buf);
	let headers = res.headers_mut();
	headers.insert(CACHE_CONTROL, HeaderValue::from_static("public,max-age=2592000"));
	headers.insert(
		CONTENT_SECURITY_POLICY,
		format!(
			"default-src 'none';base-uri 'none';script-src 'unsafe-inline' https://cdn.jsdelivr.net 'sha256-{}' 'strict-dynamic';style-src 'unsafe-inline' https://fonts.googleapis.com;font-src https://fonts.gstatic.com;connect-src 'self';img-src blob: data:",
			script_hash
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
