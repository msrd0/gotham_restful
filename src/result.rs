use crate::{ResourceType, StatusCode};
#[cfg(feature = "openapi")]
use openapiv3::SchemaKind;
use serde::Serialize;
use serde_json::error::Error as SerdeJsonError;
use std::error::Error;

/// A trait provided to convert a resource's result to json.
pub trait ResourceResult
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>;
	
	#[cfg(feature = "openapi")]
	fn schema_name() -> Option<String>;
	
	#[cfg(feature = "openapi")]
	fn to_schema() -> SchemaKind;
}

#[cfg(feature = "openapi")]
impl<Res : ResourceResult> crate::OpenapiType for Res
{
	fn schema_name() -> Option<String>
	{
		Self::schema_name()
	}

	fn to_schema() -> SchemaKind
	{
		Self::to_schema()
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
	fn schema_name() -> Option<String>
	{
		R::schema_name()
	}
	
	#[cfg(feature = "openapi")]
	fn to_schema() -> SchemaKind
	{
		R::to_schema()
	}
}

/// This can be returned from a resource when there is no cause of an error.
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
	fn schema_name() -> Option<String>
	{
		T::schema_name()
	}
	
	#[cfg(feature = "openapi")]
	fn to_schema() -> SchemaKind
	{
		T::to_schema()
	}
}
