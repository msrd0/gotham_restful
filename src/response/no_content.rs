use super::{handle_error, IntoResponse};
#[cfg(feature = "openapi")]
use crate::ResponseSchema;
use crate::{IntoResponseError, Response};
use futures_util::{future, future::FutureExt};
use gotham::hyper::header::{HeaderMap, HeaderValue, IntoHeaderName};
#[cfg(feature = "openapi")]
use gotham::hyper::StatusCode;
use mime::Mime;
#[cfg(feature = "openapi")]
use openapi_type::{OpenapiSchema, OpenapiType};
use std::{fmt::Display, future::Future, pin::Pin};

/**
This is the return type of a resource that doesn't actually return something. It will result
in a _204 No Content_ answer by default. You don't need to use this type directly if using
the function attributes:

```
# #[macro_use] extern crate gotham_restful_derive;
# mod doc_tests_are_broken {
# use gotham::state::State;
# use gotham_restful::*;
#
# #[derive(Resource)]
# #[resource(read_all)]
# struct MyResource;
#
#[read_all]
fn read_all() {
	// do something
}
# }
```
*/
#[derive(Clone, Debug, Default)]
pub struct NoContent {
	headers: HeaderMap
}

impl From<()> for NoContent {
	fn from(_: ()) -> Self {
		Self::default()
	}
}

impl NoContent {
	/// Set a custom HTTP header. If a header with this name was set before, its value is being updated.
	pub fn header<K: IntoHeaderName>(&mut self, name: K, value: HeaderValue) {
		self.headers.insert(name, value);
	}

	/// Allow manipulating HTTP headers.
	pub fn headers_mut(&mut self) -> &mut HeaderMap {
		&mut self.headers
	}
}

impl IntoResponse for NoContent {
	// TODO this shouldn't be a serde_json::Error
	type Err = serde_json::Error; // just for easier handling of `Result<NoContent, E>`

	/// This will always be a _204 No Content_ together with an empty string.
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>> {
		future::ok(Response::no_content().with_headers(self.headers)).boxed()
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		Some(Vec::new())
	}
}

#[cfg(feature = "openapi")]
impl ResponseSchema for NoContent {
	/// Returns the schema of the `()` type.
	fn schema() -> OpenapiSchema {
		<()>::schema()
	}

	/// This will always be a _204 No Content_
	fn default_status() -> StatusCode {
		StatusCode::NO_CONTENT
	}
}

impl<E> IntoResponse for Result<NoContent, E>
where
	E: Display + IntoResponseError<Err = serde_json::Error>
{
	type Err = serde_json::Error;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, serde_json::Error>> + Send>> {
		match self {
			Ok(nc) => nc.into_response(),
			Err(e) => handle_error(e)
		}
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		NoContent::accepted_types()
	}
}

#[cfg(feature = "openapi")]
impl<E> ResponseSchema for Result<NoContent, E>
where
	E: Display + IntoResponseError<Err = serde_json::Error>
{
	fn schema() -> OpenapiSchema {
		<NoContent as ResponseSchema>::schema()
	}

	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode {
		NoContent::default_status()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_executor::block_on;
	use gotham::hyper::{header::ACCESS_CONTROL_ALLOW_ORIGIN, StatusCode};
	use thiserror::Error;

	#[derive(Debug, Default, Error)]
	#[error("An Error")]
	struct MsgError;

	#[test]
	fn no_content_has_empty_response() {
		let no_content = NoContent::default();
		let res = block_on(no_content.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::NO_CONTENT);
		assert_eq!(res.mime, None);
		assert_eq!(res.full_body().unwrap(), &[] as &[u8]);

		#[cfg(feature = "openapi")]
		assert_eq!(NoContent::default_status(), StatusCode::NO_CONTENT);
	}

	#[test]
	fn no_content_result() {
		let no_content: Result<NoContent, MsgError> = Ok(NoContent::default());
		let res = block_on(no_content.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::NO_CONTENT);
		assert_eq!(res.mime, None);
		assert_eq!(res.full_body().unwrap(), &[] as &[u8]);
	}

	#[test]
	fn no_content_custom_headers() {
		let mut no_content = NoContent::default();
		no_content.header(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
		let res = block_on(no_content.into_response()).expect("didn't expect error response");
		let cors = res.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN);
		assert_eq!(cors.map(|value| value.to_str().unwrap()), Some("*"));
	}
}
