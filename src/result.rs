use crate::StatusCode;
use serde::Serialize;
use std::error::Error;

pub trait ResourceResult<R : Serialize, E : Serialize>
{
	fn to_result(self) -> (StatusCode, Result<R, E>);
}

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

impl<R : Serialize, E : Error> ResourceResult<R, ResourceError> for Result<R, E>
{
	fn to_result(self) -> (StatusCode, Result<R, ResourceError>)
	{
		match self {
			Ok(r) => (StatusCode::OK, Ok(r)),
			Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Err(e.into()))
		}
	}
}
