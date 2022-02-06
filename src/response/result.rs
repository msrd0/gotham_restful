use super::{handle_error, IntoResponse, ResourceError};
#[cfg(feature = "openapi")]
use crate::{MimeAndSchema, ResponseSchema};
use crate::{Response, ResponseBody, Success};
use futures_core::future::Future;
use gotham::{
	anyhow,
	hyper::StatusCode,
	mime::{Mime, APPLICATION_JSON}
};
use std::{fmt::Debug, pin::Pin};

pub trait IntoResponseError {
	type Err: Debug + Send + 'static;

	fn into_response_error(self) -> Result<Response, Self::Err>;
}

impl<E> IntoResponseError for E
where
	E: Into<anyhow::Error>
{
	type Err = serde_json::Error;

	fn into_response_error(self) -> Result<Response, Self::Err> {
		let err = ResourceError::from(self.into());
		Ok(Response::json(
			StatusCode::INTERNAL_SERVER_ERROR,
			serde_json::to_string(&err)?
		))
	}
}

impl<R, E> IntoResponse for Result<R, E>
where
	R: ResponseBody,
	E: Debug + IntoResponseError<Err = <Success<R> as IntoResponse>::Err>
{
	type Err = E::Err;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, E::Err>> + Send>> {
		match self {
			Ok(r) => Success::from(r).into_response(),
			Err(e) => handle_error(e)
		}
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		Some(vec![APPLICATION_JSON])
	}
}

#[cfg(feature = "openapi")]
impl<R, E> ResponseSchema for Result<R, E>
where
	R: ResponseBody,
	E: Debug + IntoResponseError<Err = serde_json::Error> + ResponseSchema
{
	fn status_codes() -> Vec<StatusCode> {
		let mut status_codes = E::status_codes();
		status_codes.push(StatusCode::OK);
		status_codes
	}

	fn schema(code: StatusCode) -> Vec<MimeAndSchema> {
		match code {
			StatusCode::OK => vec![MimeAndSchema {
				mime: APPLICATION_JSON,
				schema: R::schema()
			}],
			code => E::schema(code)
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::response::OrAllTypes;
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
		assert!(
			<Result<Msg, MsgError>>::accepted_types()
				.or_all_types()
				.contains(&APPLICATION_JSON)
		)
	}
}
