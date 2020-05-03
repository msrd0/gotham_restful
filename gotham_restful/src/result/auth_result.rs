use gotham_restful_derive::ResourceError;


/**
This is an error type that always yields a _403 Forbidden_ response. This type is best used in
combination with [`AuthSuccess`] or [`AuthResult`].

  [`AuthSuccess`]: type.AuthSuccess.html
  [`AuthResult`]: type.AuthResult.html
*/
#[derive(Debug, Clone, Copy, ResourceError)]
pub enum AuthError
{
	#[status(FORBIDDEN)]
	#[display("Forbidden")]
	Forbidden
}

/**
This return type can be used to map another `ResourceResult` that can only be returned if the
client is authenticated. Otherwise, an empty _403 Forbidden_ response will be issued. Use can
look something like this (assuming the `auth` feature is enabled):

```
# #[macro_use] extern crate gotham_restful_derive;
# mod doc_tests_are_broken {
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
fn read_all(auth : AuthStatus<MyAuthData>) -> AuthSuccess<NoContent> {
	let auth_data = match auth {
		AuthStatus::Authenticated(data) => data,
		_ => return Err(Forbidden)
	};
	// do something
	Ok(NoContent::default())
}
# }
```
*/
pub type AuthSuccess<T> = Result<T, AuthError>;

/**
This is an error type that either yields a _403 Forbidden_ respone if produced from an authentication
error, or delegates to another error type. This type is best used with [`AuthResult`].

  [`AuthResult`]: type.AuthResult.html
*/
#[derive(Debug, ResourceError)]
pub enum AuthErrorOrOther<E>
{
	#[status(UNAUTHORIZED)]
	#[display("Forbidden")]
	Forbidden,
	#[display("{0}")]
	Other(#[from] E)
}

impl<E> From<AuthError> for AuthErrorOrOther<E>
{
	fn from(err : AuthError) -> Self
	{
		match err {
			AuthError::Forbidden => Self::Forbidden
		}
	}
}

/**
This return type can be used to map another `ResourceResult` that can only be returned if the
client is authenticated. Otherwise, an empty _403 Forbidden_ response will be issued. Use can
look something like this (assuming the `auth` feature is enabled):

```
# #[macro_use] extern crate gotham_restful_derive;
# mod doc_tests_are_broken {
# use gotham::state::State;
# use gotham_restful::*;
# use serde::Deserialize;
# use std::io;
#
# #[derive(Resource)]
# struct MyResource;
#
# #[derive(Clone, Deserialize)]
# struct MyAuthData { exp : u64 }
#
#[rest_read_all(MyResource)]
fn read_all(auth : AuthStatus<MyAuthData>) -> AuthResult<NoContent, io::Error> {
	let auth_data = match auth {
		AuthStatus::Authenticated(data) => data,
		_ => Err(Forbidden)?
	};
	// do something
	Ok(NoContent::default().into())
}
# }
*/
pub type AuthResult<T, E> = Result<T, AuthErrorOrOther<E>>;
