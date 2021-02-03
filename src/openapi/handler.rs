use super::SECURITY_NAME;

use futures_util::{future, future::FutureExt};
use gotham::{
	anyhow,
	handler::{Handler, HandlerFuture, NewHandler},
	helpers::http::response::create_response,
	hyper::StatusCode,
	state::State
};
use indexmap::IndexMap;
use mime::{APPLICATION_JSON, TEXT_PLAIN};
use openapiv3::{APIKeyLocation, OpenAPI, ReferenceOr, SecurityScheme};
use std::{
	pin::Pin,
	sync::{Arc, RwLock}
};

#[derive(Clone)]
pub struct OpenapiHandler {
	openapi: Arc<RwLock<OpenAPI>>
}

impl OpenapiHandler {
	pub fn new(openapi: Arc<RwLock<OpenAPI>>) -> Self {
		Self { openapi }
	}
}

impl NewHandler for OpenapiHandler {
	type Instance = Self;

	fn new_handler(&self) -> anyhow::Result<Self> {
		Ok(self.clone())
	}
}

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
fn get_security(state: &mut State) -> (Vec<SecurityRequirement>, IndexMap<String, ReferenceOr<SecurityScheme>>) {
	Default::default()
}

impl Handler for OpenapiHandler {
	fn handle(self, mut state: State) -> Pin<Box<HandlerFuture>> {
		let openapi = match self.openapi.read() {
			Ok(openapi) => openapi,
			Err(e) => {
				error!("Unable to acquire read lock for the OpenAPI specification: {}", e);
				let res = create_response(&state, StatusCode::INTERNAL_SERVER_ERROR, TEXT_PLAIN, "");
				return future::ok((state, res)).boxed();
			}
		};

		let mut openapi = openapi.clone();
		let security_schemes = get_security(&mut state);
		let mut components = openapi.components.unwrap_or_default();
		components.security_schemes = security_schemes;
		openapi.components = Some(components);

		match serde_json::to_string(&openapi) {
			Ok(body) => {
				let res = create_response(&state, StatusCode::OK, APPLICATION_JSON, body);
				future::ok((state, res)).boxed()
			},
			Err(e) => {
				error!("Unable to handle OpenAPI request due to error: {}", e);
				let res = create_response(&state, StatusCode::INTERNAL_SERVER_ERROR, TEXT_PLAIN, "");
				future::ok((state, res)).boxed()
			}
		}
	}
}
