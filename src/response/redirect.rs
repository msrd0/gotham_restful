use super::{handle_error, IntoResponse};
use crate::{IntoResponseError, Response};
#[cfg(feature = "openapi")]
use crate::{NoContent, ResponseSchema};
use futures_util::future::{BoxFuture, FutureExt, TryFutureExt};
use gotham::hyper::{
	header::{InvalidHeaderValue, LOCATION},
	Body, StatusCode
};
#[cfg(feature = "openapi")]
use openapi_type::OpenapiSchema;
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

impl IntoResponse for Redirect {
	type Err = InvalidHeaderValue;

	fn into_response(self) -> BoxFuture<'static, Result<Response, Self::Err>> {
		async move {
			let mut res = Response::new(StatusCode::SEE_OTHER, Body::empty(), None);
			res.header(LOCATION, self.to.parse()?);
			Ok(res)
		}
		.boxed()
	}
}

#[cfg(feature = "openapi")]
impl ResponseSchema for Redirect {
	fn status_codes() -> Vec<StatusCode> {
		vec![StatusCode::SEE_OTHER]
	}

	fn schema(code: StatusCode) -> OpenapiSchema {
		assert_eq!(code, StatusCode::SEE_OTHER);
		<NoContent as ResponseSchema>::schema(StatusCode::NO_CONTENT)
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
impl<E> IntoResponse for Result<Redirect, E>
where
	E: Display + IntoResponseError,
	<E as IntoResponseError>::Err: StdError + Sync
{
	type Err = RedirectError<<E as IntoResponseError>::Err>;

	fn into_response(self) -> BoxFuture<'static, Result<Response, Self::Err>> {
		match self {
			Ok(nc) => nc.into_response().map_err(Into::into).boxed(),
			Err(e) => handle_error(e).map_err(RedirectError::Other).boxed()
		}
	}
}

#[cfg(feature = "openapi")]
impl<E> ResponseSchema for Result<Redirect, E>
where
	E: Display + IntoResponseError,
	<E as IntoResponseError>::Err: StdError + Sync
{
	fn status_codes() -> Vec<StatusCode> {
		vec![StatusCode::SEE_OTHER, StatusCode::INTERNAL_SERVER_ERROR]
	}

	fn schema(code: StatusCode) -> OpenapiSchema {
		use openapiv3::{AnySchema, SchemaKind};

		match code {
			StatusCode::SEE_OTHER => <Redirect as ResponseSchema>::schema(code),
			StatusCode::INTERNAL_SERVER_ERROR => OpenapiSchema::new(SchemaKind::Any(AnySchema::default())),
			_ => panic!("Invalid status code")
		}
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
	fn rediect_response() {
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

		#[cfg(feature = "openapi")]
		assert_eq!(Redirect::status_codes(), vec![StatusCode::SEE_OTHER]);
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

		#[cfg(feature = "openapi")]
		assert_eq!(<Result<Redirect, MsgError>>::status_codes(), vec![
			StatusCode::SEE_OTHER,
			StatusCode::INTERNAL_SERVER_ERROR
		]);
	}
}
