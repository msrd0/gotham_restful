use crate::{RequestBody, ResourceResult};
use futures_util::future::BoxFuture;
use gotham::{
	extractor::{PathExtractor, QueryStringExtractor},
	hyper::{Body, Method},
	state::State
};
use std::borrow::Cow;

// TODO: Specify default types once https://github.com/rust-lang/rust/issues/29661 lands.
#[_private_openapi_trait(EndpointWithSchema)]
pub trait Endpoint {
	/// The HTTP Verb of this endpoint.
	fn http_method() -> Method;
	/// The URI that this endpoint listens on in gotham's format.
	fn uri() -> Cow<'static, str>;

	/// The output type that provides the response.
	#[openapi_bound("Output: crate::ResourceResultSchema")]
	type Output: ResourceResult + Send;

	/// Returns `true` _iff_ the URI contains placeholders. `false` by default.
	fn has_placeholders() -> bool {
		false
	}
	/// The type that parses the URI placeholders. Use [gotham::extractor::NoopPathExtractor]
	/// if `has_placeholders()` returns `false`.
	#[openapi_bound("Placeholders: crate::OpenapiType")]
	type Placeholders: PathExtractor<Body> + Sync;

	/// Returns `true` _iff_ the request parameters should be parsed. `false` by default.
	fn needs_params() -> bool {
		false
	}
	/// The type that parses the request parameters. Use [gotham::extractor::NoopQueryStringExtractor]
	/// if `needs_params()` returns `false`.
	#[openapi_bound("Params: crate::OpenapiType")]
	type Params: QueryStringExtractor<Body> + Sync;

	/// Returns `true` _iff_ the request body should be parsed. `false` by default.
	fn needs_body() -> bool {
		false
	}
	/// The type to parse the body into. Use `()` if `needs_body()` returns `false`.
	type Body: RequestBody + Send;

	/// Returns `true` if the request wants to know the auth status of the client. `false` by default.
	fn wants_auth() -> bool {
		false
	}

	/// Replace the automatically generated operation id with a custom one. Only relevant for the
	/// OpenAPI Specification.
	#[openapi_only]
	fn operation_id() -> Option<String> {
		None
	}

	/// The handler for this endpoint.
	fn handle<'a>(
		state: &'a mut State,
		placeholders: Self::Placeholders,
		params: Self::Params,
		body: Option<Self::Body>
	) -> BoxFuture<'a, Self::Output>;
}

#[cfg(feature = "openapi")]
impl<E: EndpointWithSchema> Endpoint for E {
	fn http_method() -> Method {
		E::http_method()
	}
	fn uri() -> Cow<'static, str> {
		E::uri()
	}

	type Output = E::Output;

	fn has_placeholders() -> bool {
		E::has_placeholders()
	}
	type Placeholders = E::Placeholders;

	fn needs_params() -> bool {
		E::needs_params()
	}
	type Params = E::Params;

	fn needs_body() -> bool {
		E::needs_body()
	}
	type Body = E::Body;

	fn wants_auth() -> bool {
		E::wants_auth()
	}

	fn handle<'a>(
		state: &'a mut State,
		placeholders: Self::Placeholders,
		params: Self::Params,
		body: Option<Self::Body>
	) -> BoxFuture<'a, Self::Output> {
		E::handle(state, placeholders, params, body)
	}
}
