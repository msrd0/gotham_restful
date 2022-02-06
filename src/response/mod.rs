use futures_util::future::{self, BoxFuture, FutureExt};
use gotham::{
	handler::HandlerError,
	hyper::{
		header::{HeaderMap, HeaderName, HeaderValue},
		Body, StatusCode
	},
	mime::{Mime, APPLICATION_JSON, STAR_STAR}
};
#[cfg(feature = "openapi")]
use openapi_type::{OpenapiSchema, OpenapiType};
use serde::Serialize;
use std::{
	convert::Infallible,
	fmt::{Debug, Display},
	future::Future,
	pin::Pin
};

mod auth_result;
#[allow(unreachable_pub)]
pub use auth_result::{AuthError, AuthErrorOrOther, AuthResult, AuthSuccess};

mod no_content;
#[allow(unreachable_pub)]
pub use no_content::NoContent;

mod raw;
#[allow(unreachable_pub)]
pub use raw::Raw;

mod redirect;
#[allow(unreachable_pub)]
pub use redirect::Redirect;

mod result;
#[allow(unreachable_pub)]
pub use result::IntoResponseError;

mod success;
#[allow(unreachable_pub)]
pub use success::Success;

pub(crate) trait OrAllTypes {
	fn or_all_types(self) -> Vec<Mime>;
}

impl OrAllTypes for Option<Vec<Mime>> {
	fn or_all_types(self) -> Vec<Mime> {
		self.unwrap_or_else(|| vec![STAR_STAR])
	}
}

/// A response, used to create the final gotham response from.
///
/// This type is not meant to be used as the return type of endpoint handlers. While it can be
/// freely used without the `openapi` feature, it is more complicated to use when you enable it,
/// since this type does not store any schema information. You can attach schema information
/// like so:
///
/// ```rust
/// # #[cfg(feature = "openapi")] mod example {
/// # use gotham::hyper::StatusCode;
/// # use gotham_restful::*;
/// # use openapi_type::*;
/// fn schema(code: StatusCode) -> OpenapiSchema {
/// 	assert_eq!(code, StatusCode::ACCEPTED);
/// 	<()>::schema()
/// }
///
/// fn status_codes() -> Vec<StatusCode> {
/// 	vec![StatusCode::ACCEPTED]
/// }
///
/// #[create(schema = "schema", status_codes = "status_codes")]
/// fn create(body: Raw<Vec<u8>>) {}
/// # }
/// ```
#[derive(Debug)]
pub struct Response {
	pub(crate) status: StatusCode,
	pub(crate) body: Body,
	pub(crate) mime: Option<Mime>,
	pub(crate) headers: HeaderMap
}

impl Response {
	/// Create a new [Response] from raw data.
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn new<B: Into<Body>>(status: StatusCode, body: B, mime: Option<Mime>) -> Self {
		Self {
			status,
			body: body.into(),
			mime,
			headers: Default::default()
		}
	}

	/// Create a [Response] with mime type json from already serialized data.
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn json<B: Into<Body>>(status: StatusCode, body: B) -> Self {
		Self {
			status,
			body: body.into(),
			mime: Some(APPLICATION_JSON),
			headers: Default::default()
		}
	}

	/// Create a _204 No Content_ [Response].
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn no_content() -> Self {
		Self {
			status: StatusCode::NO_CONTENT,
			body: Body::empty(),
			mime: None,
			headers: Default::default()
		}
	}

	/// Create an empty _403 Forbidden_ [Response].
	#[must_use = "Creating a response is pointless if you don't use it"]
	pub fn forbidden() -> Self {
		Self {
			status: StatusCode::FORBIDDEN,
			body: Body::empty(),
			mime: None,
			headers: Default::default()
		}
	}

	/// Return the status code of this [Response].
	pub fn status(&self) -> StatusCode {
		self.status
	}

	/// Return the mime type of this [Response].
	pub fn mime(&self) -> Option<&Mime> {
		self.mime.as_ref()
	}

	/// Add an HTTP header to the [Response].
	pub fn header(&mut self, name: HeaderName, value: HeaderValue) {
		self.headers.insert(name, value);
	}

	pub(crate) fn with_headers(mut self, headers: HeaderMap) -> Self {
		self.headers = headers;
		self
	}

	#[cfg(test)]
	pub(crate) fn full_body(
		mut self
	) -> Result<Vec<u8>, <Body as gotham::hyper::body::HttpBody>::Error> {
		use futures_executor::block_on;
		use gotham::hyper::body::to_bytes;

		let bytes: &[u8] = &block_on(to_bytes(&mut self.body))?;
		Ok(bytes.to_vec())
	}
}

impl IntoResponse for Response {
	type Err = Infallible;

	fn into_response(self) -> BoxFuture<'static, Result<Response, Self::Err>> {
		future::ok(self).boxed()
	}
}

/// This trait needs to be implemented by every type returned from an endpoint to
/// to provide the response.
pub trait IntoResponse {
	type Err: Into<HandlerError> + Send + Sync + 'static;

	/// Turn this into a response that can be returned to the browser. This api will likely
	/// change in the future.
	fn into_response(self) -> BoxFuture<'static, Result<Response, Self::Err>>;

	/// Return a list of supported mime types.
	fn accepted_types() -> Option<Vec<Mime>> {
		None
	}
}

#[cfg(feature = "openapi")]
#[derive(Debug)]
pub struct MimeAndSchema {
	pub mime: Mime,
	pub schema: OpenapiSchema
}

/// Additional details for [IntoResponse] to be used with an OpenAPI-aware router.
#[cfg(feature = "openapi")]
pub trait ResponseSchema {
	/// All status codes returned by this response. Returns `[StatusCode::OK]` by default.
	fn status_codes() -> Vec<StatusCode> {
		vec![StatusCode::OK]
	}

	/// Return the schema of the response for the given status code. The code may
	/// only be one that was previously returned by [Self::status_codes]. The
	/// implementation should panic if that is not the case.
	fn schema(code: StatusCode) -> Vec<MimeAndSchema>;
}

#[cfg(feature = "openapi")]
mod private {
	pub trait Sealed {}
}

/// A trait provided to convert a resource's result to json, and provide an OpenAPI schema to the
/// router. This trait is implemented for all types that implement [IntoResponse] and
/// [ResponseSchema].
#[cfg(feature = "openapi")]
pub trait IntoResponseWithSchema: IntoResponse + ResponseSchema + private::Sealed {}

#[cfg(feature = "openapi")]
impl<R: IntoResponse + ResponseSchema> private::Sealed for R {}

#[cfg(feature = "openapi")]
impl<R: IntoResponse + ResponseSchema> IntoResponseWithSchema for R {}

/// The default json returned on an 500 Internal Server Error.
#[derive(Debug, Serialize)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
pub(crate) struct ResourceError {
	/// This is always `true` and can be used to detect an error response without looking at the
	/// HTTP status code.
	error: bool,
	/// The error message.
	message: String
}

impl<T: ToString> From<T> for ResourceError {
	fn from(message: T) -> Self {
		Self {
			error: true,
			message: message.to_string()
		}
	}
}

#[cfg(feature = "errorlog")]
fn errorlog<E: Display>(e: E) {
	error!("The handler encountered an error: {e}");
}

#[cfg(not(feature = "errorlog"))]
fn errorlog<E>(_e: E) {}

fn handle_error<E>(e: E) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>>
where
	E: Debug + IntoResponseError
{
	let msg = format!("{e:?}");
	let res = e.into_response_error();
	match &res {
		Ok(res) if res.status.is_server_error() => errorlog(msg),
		Err(err) => {
			errorlog(msg);
			errorlog(format!("{err:?}"));
		},
		_ => {}
	};
	future::ready(res).boxed()
}

impl<Res> IntoResponse for Pin<Box<dyn Future<Output = Res> + Send>>
where
	Res: IntoResponse + 'static
{
	type Err = Res::Err;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>> {
		self.then(IntoResponse::into_response).boxed()
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		Res::accepted_types()
	}
}

#[cfg(feature = "openapi")]
impl<Res> ResponseSchema for Pin<Box<dyn Future<Output = Res> + Send>>
where
	Res: ResponseSchema
{
	fn status_codes() -> Vec<StatusCode> {
		Res::status_codes()
	}

	fn schema(code: StatusCode) -> Vec<MimeAndSchema> {
		Res::schema(code)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_executor::block_on;
	use thiserror::Error;

	#[derive(Debug, Default, Deserialize, Serialize)]
	#[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
	struct Msg {
		msg: String
	}

	#[derive(Debug, Default, Error)]
	#[error("An Error")]
	struct MsgError;

	#[test]
	fn result_from_future() {
		let nc = NoContent::default();
		let res = block_on(nc.into_response()).unwrap();

		let fut_nc = async move { NoContent::default() }.boxed();
		let fut_res = block_on(fut_nc.into_response()).unwrap();

		assert_eq!(res.status, fut_res.status);
		assert_eq!(res.mime, fut_res.mime);
		assert_eq!(res.full_body().unwrap(), fut_res.full_body().unwrap());
	}
}
