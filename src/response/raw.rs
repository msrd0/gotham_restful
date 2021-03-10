use super::{handle_error, IntoResponse, IntoResponseError};
use crate::{FromBody, RequestBody, ResourceType, Response};
#[cfg(feature = "openapi")]
use crate::{IntoResponseWithSchema, ResponseSchema};
#[cfg(feature = "openapi")]
use openapi_type::{OpenapiSchema, OpenapiType};

use futures_core::future::Future;
use futures_util::{future, future::FutureExt};
use gotham::hyper::{
	body::{Body, Bytes},
	StatusCode
};
use mime::Mime;
#[cfg(feature = "openapi")]
use openapiv3::{SchemaKind, StringFormat, StringType, Type, VariantOrUnknownOrEmpty};
use serde_json::error::Error as SerdeJsonError;
use std::{convert::Infallible, fmt::Display, pin::Pin};

/**
This type can be used both as a raw request body, as well as as a raw response. However, all types
of request bodies are accepted by this type. It is therefore recommended to derive your own type
from [RequestBody] and only use this when you need to return a raw response. This is a usage
example that simply returns its body:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::router::builder::*;
# use gotham_restful::*;
#[derive(Resource)]
#[resource(create)]
struct ImageResource;

#[create]
fn create(body : Raw<Vec<u8>>) -> Raw<Vec<u8>> {
	body
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<ImageResource>("img");
# 	}));
# }
```
*/
#[derive(Debug)]
pub struct Raw<T> {
	pub raw: T,
	pub mime: Mime
}

impl<T> Raw<T> {
	pub fn new(raw: T, mime: Mime) -> Self {
		Self { raw, mime }
	}
}

impl<T, U> AsMut<U> for Raw<T>
where
	T: AsMut<U>
{
	fn as_mut(&mut self) -> &mut U {
		self.raw.as_mut()
	}
}

impl<T, U> AsRef<U> for Raw<T>
where
	T: AsRef<U>
{
	fn as_ref(&self) -> &U {
		self.raw.as_ref()
	}
}

impl<T: Clone> Clone for Raw<T> {
	fn clone(&self) -> Self {
		Self {
			raw: self.raw.clone(),
			mime: self.mime.clone()
		}
	}
}

impl<T: for<'a> From<&'a [u8]>> FromBody for Raw<T> {
	type Err = Infallible;

	fn from_body(body: Bytes, mime: Mime) -> Result<Self, Self::Err> {
		Ok(Self::new(body.as_ref().into(), mime))
	}
}

impl<T> RequestBody for Raw<T> where Raw<T>: FromBody + ResourceType {}

#[cfg(feature = "openapi")]
impl<T> OpenapiType for Raw<T> {
	fn schema() -> OpenapiSchema {
		OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
			format: VariantOrUnknownOrEmpty::Item(StringFormat::Binary),
			..Default::default()
		})))
	}
}

impl<T: Into<Body>> IntoResponse for Raw<T>
where
	Self: Send
{
	type Err = SerdeJsonError; // just for easier handling of `Result<Raw<T>, E>`

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, SerdeJsonError>> + Send>> {
		future::ok(Response::new(StatusCode::OK, self.raw, Some(self.mime))).boxed()
	}
}

#[cfg(feature = "openapi")]
impl<T: Into<Body>> ResponseSchema for Raw<T>
where
	Self: Send
{
	fn schema() -> OpenapiSchema {
		<Self as OpenapiType>::schema()
	}
}

impl<T, E> IntoResponse for Result<Raw<T>, E>
where
	Raw<T>: IntoResponse,
	E: Display + IntoResponseError<Err = <Raw<T> as IntoResponse>::Err>
{
	type Err = E::Err;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>> {
		match self {
			Ok(raw) => raw.into_response(),
			Err(e) => handle_error(e)
		}
	}
}

#[cfg(feature = "openapi")]
impl<T, E> ResponseSchema for Result<Raw<T>, E>
where
	Raw<T>: IntoResponseWithSchema,
	E: Display + IntoResponseError<Err = <Raw<T> as IntoResponse>::Err>
{
	fn schema() -> OpenapiSchema {
		<Raw<T> as ResponseSchema>::schema()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_executor::block_on;
	use mime::TEXT_PLAIN;

	#[test]
	fn raw_response() {
		let msg = "Test";
		let raw = Raw::new(msg, TEXT_PLAIN);
		let res = block_on(raw.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(TEXT_PLAIN));
		assert_eq!(res.full_body().unwrap(), msg.as_bytes());
	}
}