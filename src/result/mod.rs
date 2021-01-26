#[cfg(feature = "openapi")]
use crate::OpenapiSchema;
use crate::Response;
use futures_util::future::FutureExt;
use mime::{Mime, STAR_STAR};
use serde::Serialize;
use std::{
	error::Error,
	fmt::{Debug, Display},
	future::Future,
	pin::Pin
};

mod auth_result;
pub use auth_result::{AuthError, AuthErrorOrOther, AuthResult, AuthSuccess};

mod no_content;
pub use no_content::NoContent;

mod raw;
pub use raw::Raw;

mod redirect;
pub use redirect::Redirect;

#[allow(clippy::module_inception)]
mod result;
pub use result::IntoResponseError;

mod success;
pub use success::Success;

pub(crate) trait OrAllTypes {
	fn or_all_types(self) -> Vec<Mime>;
}

impl OrAllTypes for Option<Vec<Mime>> {
	fn or_all_types(self) -> Vec<Mime> {
		self.unwrap_or_else(|| vec![STAR_STAR])
	}
}

/// A trait provided to convert a resource's result to json.
pub trait ResourceResult {
	type Err: Error + Send + Sync + 'static;

	/// Turn this into a response that can be returned to the browser. This api will likely
	/// change in the future.
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>>;

	/// Return a list of supported mime types.
	fn accepted_types() -> Option<Vec<Mime>> {
		None
	}

	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema;

	#[cfg(feature = "openapi")]
	fn default_status() -> crate::StatusCode {
		crate::StatusCode::OK
	}
}

#[cfg(feature = "openapi")]
impl<Res: ResourceResult> crate::OpenapiType for Res {
	fn schema() -> OpenapiSchema {
		Self::schema()
	}
}

/// The default json returned on an 500 Internal Server Error.
#[derive(Debug, Serialize)]
pub(crate) struct ResourceError {
	error: bool,
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

fn into_response_helper<Err, F>(create_response: F) -> Pin<Box<dyn Future<Output = Result<Response, Err>> + Send>>
where
	Err: Send + 'static,
	F: FnOnce() -> Result<Response, Err>
{
	let res = create_response();
	async move { res }.boxed()
}

#[cfg(feature = "errorlog")]
fn errorlog<E: Display>(e: E) {
	error!("The handler encountered an error: {}", e);
}

#[cfg(not(feature = "errorlog"))]
fn errorlog<E>(_e: E) {}

fn handle_error<E>(e: E) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>>
where
	E: Display + IntoResponseError
{
	into_response_helper(|| {
		let msg = e.to_string();
		let res = e.into_response_error();
		match &res {
			Ok(res) if res.status.is_server_error() => errorlog(msg),
			Err(err) => {
				errorlog(msg);
				errorlog(&err);
			},
			_ => {}
		};
		res
	})
}

impl<Res> ResourceResult for Pin<Box<dyn Future<Output = Res> + Send>>
where
	Res: ResourceResult + 'static
{
	type Err = Res::Err;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>> {
		self.then(ResourceResult::into_response).boxed()
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		Res::accepted_types()
	}

	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema {
		Res::schema()
	}

	#[cfg(feature = "openapi")]
	fn default_status() -> crate::StatusCode {
		Res::default_status()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_executor::block_on;
	use thiserror::Error;

	#[derive(Debug, Default, Deserialize, Serialize)]
	#[cfg_attr(feature = "openapi", derive(crate::OpenapiType))]
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
