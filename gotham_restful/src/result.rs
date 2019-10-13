use crate::{ResourceType, StatusCode};
#[cfg(feature = "openapi")]
use crate::{OpenapiSchema, OpenapiType};
use serde::Serialize;
use serde_json::error::Error as SerdeJsonError;
use std::error::Error;

/// A trait provided to convert a resource's result to json.
pub trait ResourceResult
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>;
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema;
	
	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode
	{
		StatusCode::OK
	}
}

#[cfg(feature = "openapi")]
impl<Res : ResourceResult> crate::OpenapiType for Res
{
	fn schema() -> OpenapiSchema
	{
		Self::schema()
	}
}

/// The default json returned on an 500 Internal Server Error.
#[derive(Debug, Serialize)]
pub struct ResourceError
{
	error : bool,
	message : String
}

impl<T : ToString> From<T> for ResourceError
{
	fn from(message : T) -> Self
	{
		Self {
			error: true,
			message: message.to_string()
		}
	}
}

impl<R : ResourceType, E : Error> ResourceResult for Result<R, E>
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>
	{
		Ok(match self {
			Ok(r) => (StatusCode::OK, serde_json::to_string(r)?),
			Err(e) => {
				let err : ResourceError = e.into();
				(StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&err)?)
			}
		})
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		R::schema()
	}
}

/**
This can be returned from a resource when there is no cause of an error. For example:

```
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::state::State;
# use gotham_restful::Success;
# use serde::{Deserialize, Serialize};
#
# #[derive(Resource)]
# struct MyResource;
#
#[derive(Deserialize, Serialize)]
# #[derive(OpenapiType)]
struct MyResponse {
	message: String
}

#[rest_read_all(MyResource)]
fn read_all(_state: &mut State) -> Success<MyResponse> {
	let res = MyResponse { message: "I'm always happy".to_string() };
	res.into()
}
```
*/
pub struct Success<T>(T);

impl<T> From<T> for Success<T>
{
	fn from(t : T) -> Self
	{
		Self(t)
	}
}

impl<T : ResourceType> ResourceResult for Success<T>
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>
	{
		Ok((StatusCode::OK, serde_json::to_string(&self.0)?))
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		T::schema()
	}
}

/**
This is the return type of a resource that doesn't actually return something. It will result
in a _204 No Content_ answer by default. You don't need to use this type directly if using
the function attributes:

```
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::state::State;
#
# #[derive(Resource)]
# struct MyResource;
#
#[rest_read_all(MyResource)]
fn read_all(_state: &mut State) {
	// do something
}
```
*/
#[derive(Default)]
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
	/// This will always be a _204 No Content_ together with an empty string.
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>
	{
		Ok((Self::default_status(), "".to_string()))
	}
	
	/// Returns the schema of the `()` type.
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		<()>::schema()
	}
	
	/// This will always be a _204 No Content_
	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode
	{
		StatusCode::NO_CONTENT
	}
}

impl<E : Error> ResourceResult for Result<NoContent, E>
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>
	{
		Ok(match self {
			Ok(_) => (StatusCode::NO_CONTENT, "".to_string()),
			Err(e) => {
				let err : ResourceError = e.into();
				(StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&err)?)
			}
		})
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		<()>::schema()
	}
	
	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode
	{
		StatusCode::NO_CONTENT
	}
}
