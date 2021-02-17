use super::{handle_error, ResourceResult};
use crate::{IntoResponseError, Response};
#[cfg(feature = "openapi")]
use crate::{NoContent, OpenapiSchema};
use futures_util::future::{BoxFuture, FutureExt, TryFutureExt};
use gotham::hyper::{
	header::{InvalidHeaderValue, LOCATION},
	Body, StatusCode
};
use std::{
	error::Error as StdError,
	fmt::{Debug, Display}
};
use thiserror::Error;

/**
This is the return type of a resource that only returns a redirect. It will result
in a _303 See Other_ answer, meaning the redirect will always result in a GET request
on the target.

```
# #[macro_use] extern crate gotham_restful_derive;
# mod doc_tests_are_broken {
# use gotham::state::State;
# use gotham_restful::*;
#
# #[derive(Resource)]
# #[resource(read_all)]
# struct MyResource;
#
#[read_all]
fn read_all() -> Redirect {
	Redirect {
		to: "http://localhost:8080/cool/new/location".to_owned()
	}
}
# }
```
*/
#[derive(Clone, Debug, Default)]
pub struct Redirect {
	pub to: String
}

impl ResourceResult for Redirect {
	type Err = InvalidHeaderValue;

	fn into_response(self) -> BoxFuture<'static, Result<Response, Self::Err>> {
		async move {
			let mut res = Response::new(StatusCode::SEE_OTHER, Body::empty(), None);
			res.header(LOCATION, self.to.parse()?);
			Ok(res)
		}
		.boxed()
	}

	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode {
		StatusCode::SEE_OTHER
	}

	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema {
		<NoContent as ResourceResult>::schema()
	}
}

// private type due to parent mod
#[derive(Debug, Error)]
pub enum RedirectError<E: StdError + 'static> {
	#[error("{0}")]
	InvalidLocation(#[from] InvalidHeaderValue),
	#[error("{0}")]
	Other(#[source] E)
}

#[allow(ambiguous_associated_items)] // an enum variant is not a type. never.
impl<E> ResourceResult for Result<Redirect, E>
where
	E: Display + IntoResponseError,
	<E as IntoResponseError>::Err: StdError + Sync
{
	type Err = RedirectError<<E as IntoResponseError>::Err>;

	fn into_response(self) -> BoxFuture<'static, Result<Response, Self::Err>> {
		match self {
			Ok(nc) => nc.into_response().map_err(Into::into).boxed(),
			Err(e) => handle_error(e).map_err(|e| RedirectError::Other(e)).boxed()
		}
	}

	#[cfg(feature = "openapi")]
	fn default_status() -> StatusCode {
		Redirect::default_status()
	}

	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema {
		<Redirect as ResourceResult>::schema()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use futures_executor::block_on;
	use gotham::hyper::StatusCode;
	use thiserror::Error;

	#[derive(Debug, Default, Error)]
	#[error("An Error")]
	struct MsgError;

	#[test]
	fn rediect_has_redirect_response() {
		let redir = Redirect {
			to: "http://localhost/foo".to_owned()
		};
		let res = block_on(redir.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::SEE_OTHER);
		assert_eq!(res.mime, None);
		assert_eq!(
			res.headers.get(LOCATION).map(|hdr| hdr.to_str().unwrap()),
			Some("http://localhost/foo")
		);
		assert_eq!(res.full_body().unwrap(), &[] as &[u8]);
	}

	#[test]
	fn redirect_result() {
		let redir: Result<Redirect, MsgError> = Ok(Redirect {
			to: "http://localhost/foo".to_owned()
		});
		let res = block_on(redir.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::SEE_OTHER);
		assert_eq!(res.mime, None);
		assert_eq!(
			res.headers.get(LOCATION).map(|hdr| hdr.to_str().unwrap()),
			Some("http://localhost/foo")
		);
		assert_eq!(res.full_body().unwrap(), &[] as &[u8]);
	}
}