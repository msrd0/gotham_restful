use super::{ResourceResult, handle_error, into_response_helper};
use crate::{
	result::ResourceError,
	Response, ResponseBody, StatusCode
};
#[cfg(feature = "openapi")]
use crate::OpenapiSchema;
use futures_core::future::Future;
use mime::{Mime, APPLICATION_JSON};
use std::{
	error::Error,
	fmt::Display,
	pin::Pin
};

pub trait IntoResponseError
{
	type Err : Error + Send + 'static;
	
	fn into_response_error(self) -> Result<Response, Self::Err>;
}

impl<E : Error> IntoResponseError for E
{
	type Err = serde_json::Error;
	
	fn into_response_error(self) -> Result<Response, Self::Err>
	{
		let err : ResourceError = self.into();
		Ok(Response::json(StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&err)?))
	}
}

impl<R, E> ResourceResult for Result<R, E>
where
	R : ResponseBody,
	E : Display + IntoResponseError<Err = serde_json::Error>
{
	type Err = E::Err;
	
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>>
	{
		match self {
			Ok(r) => into_response_helper(|| Ok(Response::json(StatusCode::OK, serde_json::to_string(&r)?))),
			Err(e) => handle_error(e)
		}
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		Some(vec![APPLICATION_JSON])
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		R::schema()
	}
}