use crate::{ResponseBody, StatusCode};
#[cfg(feature = "openapi")]
use crate::{OpenapiSchema, OpenapiType};
use hyper::Body;
use mime::{Mime, APPLICATION_JSON, STAR_STAR};
#[cfg(feature = "openapi")]
use openapiv3::{SchemaKind, StringFormat, StringType, Type, VariantOrUnknownOrEmpty};
use serde::Serialize;
use serde_json::error::Error as SerdeJsonError;
use std::{
	error::Error,
	fmt::Debug
};

/// A response, used to create the final gotham response from.
pub struct Response
{
	pub status : StatusCode,
	pub body : Body,
	pub mime : Option<Mime>
}

impl Response
{
	/// Create a new `Response` from raw data.
	pub fn new<B : Into<Body>>(status : StatusCode, body : B, mime : Option<Mime>) -> Self
	{
		Self {
			status,
			body: body.into(),
			mime
		}
	}
	
	/// Create a `Response` with mime type json from already serialized data.
	pub fn json<B : Into<Body>>(status : StatusCode, body : B) -> Self
	{
		Self {
			status,
			body: body.into(),
			mime: Some(APPLICATION_JSON)
		}
	}
	
	/// Create a _204 No Content_ `Response`.
	pub fn no_content() -> Self
	{
		Self {
			status: StatusCode::NO_CONTENT,
			body: Body::empty(),
			mime: None
		}
	}
	
	/// Create an empty _403 Forbidden_ `Response`.
	pub fn forbidden() -> Self
	{
		Self {
			status: StatusCode::FORBIDDEN,
			body: Body::empty(),
			mime: None
		}
	}
	
	#[cfg(test)]
	fn full_body(self) -> Vec<u8>
	{
		use futures::{future::Future, stream::Stream};
		
		let bytes : &[u8] = &self.body.concat2().wait().unwrap().into_bytes();
		bytes.to_vec()
	}
}

/// A trait provided to convert a resource's result to json.
pub trait ResourceResult
{
	/// Turn this into a response that can be returned to the browser. This api will likely
	/// change in the future.
	fn into_response(self) -> Result<Response, SerdeJsonError>;
	
	/// Return a list of supported mime types.
	fn accepted_types() -> Option<Vec<Mime>>
	{
		None
	}
	
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

impl<R : ResponseBody, E : Error> ResourceResult for Result<R, E>
{
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		Ok(match self {
			Ok(r) => Response::json(StatusCode::OK, serde_json::to_string(&r)?),
			Err(e) => {
				let err : ResourceError = e.into();
				Response::json(StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&err)?)
			}
		})
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

/**
This can be returned from a resource when there is no cause of an error. For example:

```
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::state::State;
# use gotham_restful::*;
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

impl<T : Clone> Clone for Success<T>
{
	fn clone(&self) -> Self
	{
		Self(self.0.clone())
	}
}

impl<T : Debug> Debug for Success<T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Success({:?})", self.0)
	}
}

impl<T : ResponseBody> ResourceResult for Success<T>
{
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		Ok(Response::json(StatusCode::OK, serde_json::to_string(&self.0)?))
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		Some(vec![APPLICATION_JSON])
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		T::schema()
	}
}

/**
This return type can be used to map another `ResourceResult` that can only be returned if the
client is authenticated. Otherwise, an empty _403 Forbidden_ response will be issued. Use can
look something like this (assuming the `auth` feature is enabled):

```
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::state::State;
# use gotham_restful::*;
# use serde::Deserialize;
#
# #[derive(Resource)]
# struct MyResource;
#
# #[derive(Clone, Deserialize)]
# struct MyAuthData { exp : u64 }
#
#[rest_read_all(MyResource)]
fn read_all(auth : AuthStatus<MyAuthData>) -> AuthResult<NoContent> {
	let auth_data = match auth {
		AuthStatus::Authenticated(data) => data,
		_ => return AuthErr
	};
	// do something
	NoContent::default().into()
}
```
*/
pub enum AuthResult<T>
{
	Ok(T),
	AuthErr
}

impl<T> AuthResult<T>
{
	pub fn is_ok(&self) -> bool
	{
		match self {
			Self::Ok(_) => true,
			_ => false
		}
	}
	
	pub fn unwrap(self) -> T
	{
		match self {
			Self::Ok(data) => data,
			_ => panic!()
		}
	}
}

impl<T> From<T> for AuthResult<T>
{
	fn from(t : T) -> Self
	{
		Self::Ok(t)
	}
}

impl<T : Clone> Clone for AuthResult<T>
{
	fn clone(&self) -> Self
	{
		match self {
			Self::Ok(t) => Self::Ok(t.clone()),
			Self::AuthErr => Self::AuthErr
		}
	}
}

impl<T : Debug> Debug for AuthResult<T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Ok(t) => write!(f, "Ok({:?})", t),
			Self::AuthErr => write!(f, "AuthErr")
		}
	}
}

impl<T : ResourceResult> ResourceResult for AuthResult<T>
{
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		match self
		{
			Self::Ok(res) => res.into_response(),
			Self::AuthErr => Ok(Response::forbidden())
		}
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		T::accepted_types()
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		T::schema()
	}
	
	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode
	{
		T::default_status()
	}
}

/**
This is the return type of a resource that doesn't actually return something. It will result
in a _204 No Content_ answer by default. You don't need to use this type directly if using
the function attributes:

```
# #[macro_use] extern crate gotham_restful_derive;
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
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		Ok(Response::no_content())
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
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		match self {
			Ok(nc) => nc.into_response(),
			Err(e) => {
				let err : ResourceError = e.into();
				Ok(Response::json(StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&err)?))
			}
		}
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		<NoContent as ResourceResult>::schema()
	}
	
	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode
	{
		NoContent::default_status()
	}
}

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

impl<T : Debug> Debug for Raw<T>
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Raw({:?}, {:?})", self.raw, self.mime)
	}
}

impl<T : Into<Body>> ResourceResult for Raw<T>
{
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		Ok(Response::new(StatusCode::OK, self.raw, Some(self.mime.clone())))
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		Some(vec![STAR_STAR])
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

impl<T, E : Error> ResourceResult for Result<Raw<T>, E>
where
	Raw<T> : ResourceResult
{
	fn into_response(self) -> Result<Response, SerdeJsonError>
	{
		match self {
			Ok(raw) => raw.into_response(),
			Err(e) => {
				let err : ResourceError = e.into();
				Ok(Response::json(StatusCode::INTERNAL_SERVER_ERROR, serde_json::to_string(&err)?))
			}
		}
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		<Raw<T> as ResourceResult>::accepted_types()
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
	use mime::TEXT_PLAIN;
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
		let res = ok.into_response().expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body(), r#"{"msg":""}"#.as_bytes());
	}
	
	#[test]
	fn resource_result_err()
	{
		let err : Result<Msg, MsgError> = Err(MsgError::default());
		let res = err.into_response().expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body(), format!(r#"{{"error":true,"message":"{}"}}"#, MsgError::default()).as_bytes());
	}
	
	#[test]
	fn success_always_successfull()
	{
		let success : Success<Msg> = Msg::default().into();
		let res = success.into_response().expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body(), r#"{"msg":""}"#.as_bytes());
	}
	
	#[test]
	fn no_content_has_empty_response()
	{
		let no_content = NoContent::default();
		let res = no_content.into_response().expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::NO_CONTENT);
		assert_eq!(res.mime, None);
		assert_eq!(res.full_body(), &[] as &[u8]);
	}
	
	#[test]
	fn no_content_result()
	{
		let no_content : Result<NoContent, MsgError> = Ok(NoContent::default());
		let res = no_content.into_response().expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::NO_CONTENT);
		assert_eq!(res.mime, None);
		assert_eq!(res.full_body(), &[] as &[u8]);
	}
	
	#[test]
	fn raw_response()
	{
		let msg = "Test";
		let raw = Raw::new(msg, TEXT_PLAIN);
		let res = raw.into_response().expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(TEXT_PLAIN));
		assert_eq!(res.full_body(), msg.as_bytes());
	}
}
