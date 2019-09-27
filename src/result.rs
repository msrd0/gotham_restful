use crate::StatusCode;
use serde::Serialize;
use serde_json::error::Error as SerdeJsonError;
use std::error::Error;

/// A trait provided to convert a resource's result to json.
pub trait ResourceResult
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>;
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

impl<R : Serialize, E : Error> ResourceResult for Result<R, E>
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

impl<T : Serialize> ResourceResult for Success<T>
{
	fn to_json(&self) -> Result<(StatusCode, String), SerdeJsonError>
	{
		Ok((StatusCode::OK, serde_json::to_string(&self.0)?))
	}
}
