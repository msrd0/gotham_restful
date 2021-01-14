use super::{handle_error, into_response_helper, ResourceResult};
#[cfg(feature = "openapi")]
use crate::OpenapiSchema;
use crate::{result::ResourceError, Response, ResponseBody, StatusCode};
use futures_core::future::Future;
use mime::{Mime, APPLICATION_JSON};
use std::{error::Error, fmt::Display, pin::Pin};

pub trait IntoResponseError {
	type Err: Error + Send + 'static;

	fn into_response_error(self) -> Result<Response, Self::Err>;
}

impl<E: Error> IntoResponseError for E {
	type Err = serde_json::Error;

	fn into_response_error(self) -> Result<Response, Self::Err> {
		let err: ResourceError = self.into();
		Ok(Response::json(
			StatusCode::INTERNAL_SERVER_ERROR,
			serde_json::to_string(&err)?
		))
	}
}

impl<R, E> ResourceResult for Result<R, E>
where
	R: ResponseBody,
	E: Display + IntoResponseError<Err = serde_json::Error>
{
	type Err = E::Err;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>> {
		match self {
			Ok(r) => into_response_helper(|| Ok(Response::json(StatusCode::OK, serde_json::to_string(&r)?))),
			Err(e) => handle_error(e)
		}
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		Some(vec![APPLICATION_JSON])
	}

	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema {
		R::schema()
	}
}

#[cfg(test)]
#[allow(deprecated)]
mod test {
	use super::*;
	use crate::result::OrAllTypes;
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
	fn result_ok() {
		let ok: Result<Msg, MsgError> = Ok(Msg::default());
		let res = block_on(ok.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body().unwrap(), br#"{"msg":""}"#);
	}

	#[test]
	fn result_err() {
		let err: Result<Msg, MsgError> = Err(MsgError::default());
		let res = block_on(err.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::INTERNAL_SERVER_ERROR);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(
			res.full_body().unwrap(),
			format!(r#"{{"error":true,"message":"{}"}}"#, MsgError::default()).as_bytes()
		);
	}

	#[test]
	fn success_accepts_json() {
		assert!(<Result<Msg, MsgError>>::accepted_types()
			.or_all_types()
			.contains(&APPLICATION_JSON))
	}
}
