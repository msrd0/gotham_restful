use crate::{IntoResponseError, Response};
use gotham::{hyper::StatusCode, mime::TEXT_PLAIN_UTF_8};
use gotham_restful_derive::ResourceError;
#[cfg(feature = "openapi")]
use openapi_type::{OpenapiSchema, OpenapiType};

/// This is an error type that always yields a _403 Forbidden_ response. This type
/// is best used in combination with [`AuthSuccess`] or [`AuthResult`].
#[derive(Clone, Debug)]
pub struct AuthError(String);

impl AuthError {
	pub fn new<T: Into<String>>(msg: T) -> Self {
		Self(msg.into())
	}
}

impl IntoResponseError for AuthError {
	// TODO why does this need to be serde_json::Error ?!?
	type Err = serde_json::Error;

	fn into_response_error(self) -> Result<Response, Self::Err> {
		Ok(Response::new(
			StatusCode::FORBIDDEN,
			self.0,
			Some(TEXT_PLAIN_UTF_8)
		))
	}

	#[cfg(feature = "openapi")]
	fn status_codes() -> Vec<StatusCode> {
		vec![StatusCode::FORBIDDEN]
	}

	#[cfg(feature = "openapi")]
	fn schema(code: StatusCode) -> OpenapiSchema {
		assert_eq!(code, StatusCode::FORBIDDEN);
		<super::Raw<String> as OpenapiType>::schema()
	}
}

/// This return type can be used to wrap any type implementing [IntoResponse](crate::IntoResponse)
/// that can only be returned if the client is authenticated. Otherwise, an empty _403 Forbidden_
/// response will be issued.
///
/// Use can look something like this (assuming the `auth` feature is enabled):
///
/// ```rust
/// # #[macro_use] extern crate gotham_restful_derive;
/// # #[cfg(feature = "auth")]
/// # mod auth_feature_enabled {
/// # use gotham::state::State;
/// # use gotham_restful::*;
/// # use serde::Deserialize;
/// #
/// # #[derive(Resource)]
/// # #[resource(read_all)]
/// # struct MyResource;
/// #
/// # #[derive(Clone, Deserialize)]
/// # struct MyAuthData { exp : u64 }
/// #
/// #[read_all]
/// fn read_all(auth: AuthStatus<MyAuthData>) -> AuthSuccess<NoContent> {
/// 	let auth_data = auth.ok()?;
/// 	// do something
/// 	Ok(NoContent::default())
/// }
/// # }
/// ```
pub type AuthSuccess<T> = Result<T, AuthError>;

/// This is an error type that either yields a _403 Forbidden_ response if produced
/// from an authentication error, or delegates to another error type. This type is
/// best used with [`AuthResult`].
#[derive(Debug, Clone, ResourceError)]
pub enum AuthErrorOrOther<E> {
	Forbidden(#[from] AuthError),

	#[status(INTERNAL_SERVER_ERROR)]
	#[display("{0}")]
	Other(E)
}

mod private {
	use gotham::handler::HandlerError;
	pub trait Sealed {}
	impl<E: Into<HandlerError>> Sealed for E {}
}

impl<E, F> From<F> for AuthErrorOrOther<E>
where
	// TODO https://github.com/msrd0/gotham_restful/issues/20
	F: private::Sealed + Into<E>
{
	fn from(err: F) -> Self {
		Self::Other(err.into())
	}
}

/// This return type can be used to wrap any type implementing [IntoResponse](crate::IntoResponse)
/// that can only be returned if the client is authenticated. Otherwise, an empty _403 Forbidden_
/// response will be issued.
///
/// Use can look something like this (assuming the `auth` feature is enabled):
///
/// ```
/// # #[macro_use] extern crate gotham_restful_derive;
/// # #[cfg(feature = "auth")]
/// # mod auth_feature_enabled {
/// # use gotham::state::State;
/// # use gotham_restful::*;
/// # use serde::Deserialize;
/// # use std::io;
/// #
/// # #[derive(Resource)]
/// # #[resource(read_all)]
/// # struct MyResource;
/// #
/// # #[derive(Clone, Deserialize)]
/// # struct MyAuthData { exp : u64 }
/// #
/// #[read_all]
/// fn read_all(auth : AuthStatus<MyAuthData>) -> AuthResult<NoContent, io::Error> {
/// 	let auth_data = auth.ok()?;
/// do something
/// 	Ok(NoContent::default().into())
/// }
/// # }
pub type AuthResult<T, E> = Result<T, AuthErrorOrOther<E>>;
