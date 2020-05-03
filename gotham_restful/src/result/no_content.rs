use super::{ResourceResult, handle_error};
use crate::{IntoResponseError, Response};
#[cfg(feature = "openapi")]
use crate::{OpenapiSchema, OpenapiType};
use futures_util::{future, future::FutureExt};
use mime::Mime;
use std::{
	fmt::Display,
	future::Future,
	pin::Pin
};

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
# struct MyResource;
#
#[rest_read_all(MyResource)]
fn read_all(_state: &mut State) {
	// do something
}
# }
```
*/
#[derive(Clone, Copy, Debug, Default)]
pub struct NoContent;

impl From<()> for NoContent
{
	fn from(_ : ()) -> Self
	{
		Self {}
	}
}

impl ResourceResult for NoContent
{
	// TODO this shouldn't be a serde_json::Error
	type Err = serde_json::Error; // just for easier handling of `Result<NoContent, E>`
	
	/// This will always be a _204 No Content_ together with an empty string.
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>>
	{
		future::ok(Response::no_content()).boxed()
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		Some(Vec::new())
	}
	
	/// Returns the schema of the `()` type.
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		<()>::schema()
	}
	
	/// This will always be a _204 No Content_
	#[cfg(feature = "openapi")]
	fn default_status() -> crate::StatusCode
	{
		crate::StatusCode::NO_CONTENT
	}
}

impl<E> ResourceResult for Result<NoContent, E>
where
	E : Display + IntoResponseError<Err = serde_json::Error>
{
	type Err = serde_json::Error;
	
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, serde_json::Error>> + Send>>
	{
		match self {
			Ok(nc) => nc.into_response(),
			Err(e) => handle_error(e)
		}
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		NoContent::accepted_types()
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		<NoContent as ResourceResult>::schema()
	}
	
	#[cfg(feature = "openapi")]
	fn default_status() -> crate::StatusCode
	{
		NoContent::default_status()
	}
}


#[cfg(test)]
mod test
{
	use super::*;
	use futures_executor::block_on;
	use gotham::hyper::StatusCode;
	use thiserror::Error;
	
	#[derive(Debug, Default, Error)]
	#[error("An Error")]
	struct MsgError;
	
	#[test]
	fn no_content_has_empty_response()
	{
		let no_content = NoContent::default();
		let res = block_on(no_content.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::NO_CONTENT);
		assert_eq!(res.mime, None);
		assert_eq!(res.full_body().unwrap(), &[] as &[u8]);
	}
	
	#[test]
	fn no_content_result()
	{
		let no_content : Result<NoContent, MsgError> = Ok(NoContent::default());
		let res = block_on(no_content.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::NO_CONTENT);
		assert_eq!(res.mime, None);
		assert_eq!(res.full_body().unwrap(), &[] as &[u8]);
	}
}
