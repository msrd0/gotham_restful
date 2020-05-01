use crate::Response;
#[cfg(feature = "openapi")]
use crate::OpenapiSchema;
use futures_util::future::FutureExt;
use mime::Mime;
use serde::Serialize;
use std::{
	error::Error,
	future::Future,
	fmt::{Debug, Display},
	pin::Pin
};

mod auth_result;
pub use auth_result::{AuthError, AuthErrorOrOther, AuthResult, AuthSuccess};

mod no_content;
pub use no_content::NoContent;

mod raw;
pub use raw::Raw;

mod result;
pub use result::IntoResponseError;

mod success;
pub use success::Success;

/// A trait provided to convert a resource's result to json.
pub trait ResourceResult
{
	type Err : Error + Send + 'static;
	
	/// Turn this into a response that can be returned to the browser. This api will likely
	/// change in the future.
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>>;
	
	/// Return a list of supported mime types.
	fn accepted_types() -> Option<Vec<Mime>>
	{
		None
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema;
	
	#[cfg(feature = "openapi")]
	fn default_status() -> crate::StatusCode
	{
		crate::StatusCode::OK
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
pub(crate) struct ResourceError
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

fn into_response_helper<Err, F>(create_response : F) -> Pin<Box<dyn Future<Output = Result<Response, Err>> + Send>>
where
	Err : Send + 'static,
	F : FnOnce() -> Result<Response, Err>
{
	let res = create_response();
	async move { res }.boxed()
}

#[cfg(feature = "errorlog")]
fn errorlog<E : Display>(e : E)
{
	error!("The handler encountered an error: {}", e);
}

#[cfg(not(feature = "errorlog"))]
fn errorlog<E>(_e : E) {}

fn handle_error<E>(e : E) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>>
where
	E : Display + IntoResponseError
{
	into_response_helper(|| {
		errorlog(&e);
		e.into_response_error()
	})
}


impl<Res> ResourceResult for Pin<Box<dyn Future<Output = Res> + Send>>
where
	Res : ResourceResult + 'static
{
	type Err = Res::Err;
	
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>>
	{
		self.then(|result| {
			result.into_response()
		}).boxed()
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		Res::accepted_types()
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		Res::schema()
	}
	
	#[cfg(feature = "openapi")]
	fn default_status() -> crate::StatusCode
	{
		Res::default_status()
	}
}



#[cfg(test)]
mod test
{
	use super::*;
	use crate::{OpenapiType, StatusCode};
	use futures_executor::block_on;
	use mime::{APPLICATION_JSON, TEXT_PLAIN};
	use thiserror::Error;
	
	#[derive(Debug, Default, Deserialize, Serialize)]
	#[cfg_attr(feature = "openapi", derive(OpenapiType))]
	struct Msg
	{
		msg : String
	}
	
	#[derive(Debug, Default, Error)]
	#[error("An Error")]
	struct MsgError;
	
	#[test]
	fn resource_result_ok()
	{
		let ok : Result<Msg, MsgError> = Ok(Msg::default());
		let res = block_on(ok.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body().unwrap(), r#"{"msg":""}"#.as_bytes());
	}
	
	#[test]
	fn resource_result_err()
	{
		let err : Result<Msg, MsgError> = Err(MsgError::default());
		let res = block_on(err.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body().unwrap(), format!(r#"{{"error":true,"message":"{}"}}"#, MsgError::default()).as_bytes());
	}
	
	#[test]
	fn success_always_successfull()
	{
		let success : Success<Msg> = Msg::default().into();
		let res = block_on(success.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body().unwrap(), r#"{"msg":""}"#.as_bytes());
	}
	
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
