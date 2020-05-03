use super::{IntoResponseError, ResourceResult, handle_error};
use crate::{Response, StatusCode};
#[cfg(feature = "openapi")]
use crate::OpenapiSchema;
use futures_core::future::Future;
use futures_util::{future, future::FutureExt};
use gotham::hyper::Body;
use mime::Mime;
#[cfg(feature = "openapi")]
use openapiv3::{SchemaKind, StringFormat, StringType, Type, VariantOrUnknownOrEmpty};
use serde_json::error::Error as SerdeJsonError;
use std::{
	fmt::Display,
	pin::Pin
};

#[derive(Debug)]
pub struct Raw<T>
{
	pub raw : T,
	pub mime : Mime
}

impl<T> Raw<T>
{
	pub fn new(raw : T, mime : Mime) -> Self
	{
		Self { raw, mime }
	}
}

impl<T : Clone> Clone for Raw<T>
{
	fn clone(&self) -> Self
	{
		Self {
			raw: self.raw.clone(),
			mime: self.mime.clone()
		}
	}
}

impl<T : Into<Body>> ResourceResult for Raw<T>
where
	Self : Send
{
	type Err = SerdeJsonError; // just for easier handling of `Result<Raw<T>, E>`
	
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, SerdeJsonError>> + Send>>
	{
		future::ok(Response::new(StatusCode::OK, self.raw, Some(self.mime.clone()))).boxed()
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
			format: VariantOrUnknownOrEmpty::Item(StringFormat::Binary),
			..Default::default()
		})))
	}
}

impl<T, E> ResourceResult for Result<Raw<T>, E>
where
	Raw<T> : ResourceResult,
	E : Display + IntoResponseError<Err = <Raw<T> as ResourceResult>::Err>
{
	type Err = E::Err;
	
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>>
	{
		match self {
			Ok(raw) => raw.into_response(),
			Err(e) => handle_error(e)
		}
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		<Raw<T> as ResourceResult>::schema()
	}
}


#[cfg(test)]
mod test
{
	use super::*;
	use futures_executor::block_on;
	use mime::TEXT_PLAIN;
	
	#[test]
	fn raw_response()
	{
		let msg = "Test";
		let raw = Raw::new(msg, TEXT_PLAIN);
		let res = block_on(raw.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(TEXT_PLAIN));
		assert_eq!(res.full_body().unwrap(), msg.as_bytes());
	}
}
