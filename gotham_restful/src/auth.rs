use crate::HeaderName;
use cookie::CookieJar;
use futures_util::{future, future::{FutureExt, TryFutureExt}};
use gotham::{
	handler::HandlerFuture,
	hyper::header::{AUTHORIZATION, HeaderMap},
	middleware::{Middleware, NewMiddleware},
	state::{FromState, State}
};
use jsonwebtoken::{
	errors::ErrorKind,
	DecodingKey
};
use serde::de::DeserializeOwned;
use std::{
	marker::PhantomData,
	panic::RefUnwindSafe,
	pin::Pin
};

pub use jsonwebtoken::Validation as AuthValidation;

/// The authentication status returned by the auth middleware for each request.
#[derive(Debug, StateData)]
pub enum AuthStatus<T : Send + 'static>
{
	/// The auth status is unknown.
	Unknown,
	/// The request has been performed without any kind of authentication.
	Unauthenticated,
	/// The request has been performed with an invalid authentication.
	Invalid,
	/// The request has been performed with an expired authentication.
	Expired,
	/// The request has been performed with a valid authentication.
	Authenticated(T)
}

impl<T> Clone for AuthStatus<T>
where
	T : Clone + Send + 'static
{
	fn clone(&self) -> Self
	{
		match self {
			Self::Unknown => Self::Unknown,
			Self::Unauthenticated => Self::Unauthenticated,
			Self::Invalid => Self::Invalid,
			Self::Expired => Self::Expired,
			Self::Authenticated(data) => Self::Authenticated(data.clone())
		}
	}
}

/// The source of the authentication token in the request.
#[derive(Clone, StateData)]
pub enum AuthSource
{
	/// Take the token from a cookie with the given name.
	Cookie(String),
	/// Take the token from a header with the given name.
	Header(HeaderName),
	/// Take the token from the HTTP Authorization header. This is different from `Header("Authorization")`
	/// as it will follow the `scheme param` format from the HTTP specification. The `scheme` will
	/// be discarded, so its value doesn't matter.
	AuthorizationHeader
}

/**
This trait will help the auth middleware to determine the validity of an authentication token.

A very basic implementation could look like this:
```
# use gotham_restful::{AuthHandler, State};
#
const SECRET : &'static [u8; 32] = b"zlBsA2QXnkmpe0QTh8uCvtAEa4j33YAc";

struct CustomAuthHandler;
impl<T> AuthHandler<T> for CustomAuthHandler {
	fn jwt_secret<F : FnOnce() -> Option<T>>(&self, _state : &mut State, _decode_data : F) -> Option<Vec<u8>> {
		Some(SECRET.to_vec())
	}
}
```
*/
pub trait AuthHandler<Data>
{
	/// Return the SHA256-HMAC secret used to verify the JWT token.
	fn jwt_secret<F : FnOnce() -> Option<Data>>(&self, state : &mut State, decode_data : F) -> Option<Vec<u8>>;
}

/// An `AuthHandler` returning always the same secret. See `AuthMiddleware` for a usage example.
#[derive(Clone, Debug)]
pub struct StaticAuthHandler
{
	secret : Vec<u8>
}

impl StaticAuthHandler
{
	pub fn from_vec(secret : Vec<u8>) -> Self
	{
		Self { secret }
	}
	
	pub fn from_array(secret : &[u8]) -> Self
	{
		Self::from_vec(secret.to_vec())
	}
}

impl<T> AuthHandler<T> for StaticAuthHandler
{
	fn jwt_secret<F : FnOnce() -> Option<T>>(&self, _state : &mut State, _decode_data : F) -> Option<Vec<u8>>
	{
		Some(self.secret.clone())
	}
}

/**
This is the auth middleware. To use it, first make sure you have the `auth` feature enabled. Then
simply add it to your pipeline and request it inside your handler:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::{router::builder::*, pipeline::{new_pipeline, single::single_pipeline}, state::State};
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
#
#[derive(Resource)]
#[rest_resource(read_all)]
struct AuthResource;

#[derive(Debug, Deserialize, Clone)]
struct AuthData {
	sub: String,
	exp: u64
}

#[rest_read_all(AuthResource)]
fn read_all(auth : &AuthStatus<AuthData>) -> Success<String> {
	format!("{:?}", auth).into()
}

fn main() {
	let auth : AuthMiddleware<AuthData, _> = AuthMiddleware::new(
		AuthSource::AuthorizationHeader,
		AuthValidation::default(),
		StaticAuthHandler::from_array(b"zlBsA2QXnkmpe0QTh8uCvtAEa4j33YAc")
	);
	let (chain, pipelines) = single_pipeline(new_pipeline().add(auth).build());
	gotham::start("127.0.0.1:8080", build_router(chain, pipelines, |route| {
		route.resource::<AuthResource>("auth");
	}));
}
```
*/
pub struct AuthMiddleware<Data, Handler>
{
	source : AuthSource,
	validation : AuthValidation,
	handler : Handler,
	_data : PhantomData<Data>
}

impl<Data, Handler> Clone for AuthMiddleware<Data, Handler>
where Handler : Clone
{
	fn clone(&self) -> Self
	{
		Self {
			source: self.source.clone(),
			validation: self.validation.clone(),
			handler: self.handler.clone(),
			_data: self._data
		}
	}
}

impl<Data, Handler> AuthMiddleware<Data, Handler>
where
	Data : DeserializeOwned + Send,
	Handler : AuthHandler<Data> + Default
{
	pub fn from_source(source : AuthSource) -> Self
	{
		Self {
			source,
			validation: Default::default(),
			handler: Default::default(),
			_data: Default::default()
		}
	}
}

impl<Data, Handler> AuthMiddleware<Data, Handler>
where
	Data : DeserializeOwned + Send,
	Handler : AuthHandler<Data>
{
	pub fn new(source : AuthSource, validation : AuthValidation, handler : Handler) -> Self
	{
		Self {
			source,
			validation,
			handler,
			_data: Default::default()
		}
	}
	
	fn auth_status(&self, state : &mut State) -> AuthStatus<Data>
	{
		// extract the provided token, if any
		let token = match &self.source {
			AuthSource::Cookie(name) => {
				CookieJar::try_borrow_from(&state)
					.and_then(|jar| jar.get(&name))
					.map(|cookie| cookie.value().to_owned())
			},
			AuthSource::Header(name) => {
				HeaderMap::try_borrow_from(&state)
					.and_then(|map| map.get(name))
					.and_then(|header| header.to_str().ok())
					.map(|value| value.to_owned())
			},
			AuthSource::AuthorizationHeader => {
				HeaderMap::try_borrow_from(&state)
					.and_then(|map| map.get(AUTHORIZATION))
					.and_then(|header| header.to_str().ok())
					.and_then(|value| value.split_whitespace().nth(1))
					.map(|value| value.to_owned())
			}
		};
		
		// unauthed if no token
		let token = match token {
			Some(token) => token,
			None => return AuthStatus::Unauthenticated
		};
		
		// get the secret from the handler, possibly decoding claims ourselves
		let secret = self.handler.jwt_secret(state, || {
			let b64 = token.split(".").nth(1)?;
			let raw = base64::decode_config(b64, base64::URL_SAFE_NO_PAD).ok()?;
			serde_json::from_slice(&raw).ok()?
		});
		
		// unknown if no secret
		let secret = match secret {
			Some(secret) => secret,
			None => return AuthStatus::Unknown
		};
		
		// validate the token
		let data : Data = match jsonwebtoken::decode(&token, &DecodingKey::from_secret(&secret), &self.validation) {
			Ok(data) => data.claims,
			Err(e) => match dbg!(e.into_kind()) {
				ErrorKind::ExpiredSignature => return AuthStatus::Expired,
				_ => return AuthStatus::Invalid
			}
		};
		
		// we found a valid token
		return AuthStatus::Authenticated(data);
	}
}

impl<Data, Handler> Middleware for AuthMiddleware<Data, Handler>
where
	Data : DeserializeOwned + Send + 'static,
	Handler : AuthHandler<Data>
{
	fn call<Chain>(self, mut state : State, chain : Chain) -> Pin<Box<HandlerFuture>>
	where
		Chain : FnOnce(State) -> Pin<Box<HandlerFuture>>
	{
		// put the source in our state, required for e.g. openapi
		state.put(self.source.clone());
		
		// put the status in our state
		let status = self.auth_status(&mut state);
		state.put(status);
		
		// call the rest of the chain
		chain(state).and_then(|(state, res)| future::ok((state, res))).boxed()
	}
}

impl<Data, Handler> NewMiddleware for AuthMiddleware<Data, Handler>
where
	Self : Clone + Middleware + Sync + RefUnwindSafe
{
	type Instance = Self;
	
	fn new_middleware(&self) -> Result<Self::Instance, std::io::Error>
	{
		let c : Self = self.clone();
		Ok(c)
	}
}

#[cfg(test)]
mod test
{
	use super::*;
	use cookie::Cookie;
	use std::fmt::Debug;
	
	// 256-bit random string
	const JWT_SECRET : &'static [u8; 32] = b"Lyzsfnta0cdxyF0T9y6VGxp3jpgoMUuW";
	
	// some known tokens
	const VALID_TOKEN : &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJtc3JkMCIsInN1YiI6ImdvdGhhbS1yZXN0ZnVsIiwiaWF0IjoxNTc3ODM2ODAwLCJleHAiOjQxMDI0NDQ4MDB9.8h8Ax-nnykqEQ62t7CxmM3ja6NzUQ4L0MLOOzddjLKk";
	const EXPIRED_TOKEN : &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJtc3JkMCIsInN1YiI6ImdvdGhhbS1yZXN0ZnVsIiwiaWF0IjoxNTc3ODM2ODAwLCJleHAiOjE1Nzc4MzcxMDB9.eV1snaGLYrJ7qUoMk74OvBY3WUU9M0Je5HTU2xtX1v0";
	const INVALID_TOKEN : &'static str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJtc3JkMCIsInN1YiI6ImdvdGhhbS1yZXN0ZnVsIiwiaWF0IjoxNTc3ODM2ODAwLCJleHAiOjQxMDI0NDQ4MDB9";
	
	#[derive(Debug, Deserialize, PartialEq)]
	struct TestData
	{
		iss : String,
		sub : String,
		iat : u64,
		exp : u64
	}
	
	impl Default for TestData
	{
		fn default() -> Self
		{
			Self {
				iss: "msrd0".to_owned(),
				sub: "gotham-restful".to_owned(),
				iat: 1577836800,
				exp: 4102444800
			}
		}
	}
	
	#[derive(Default)]
	struct NoneAuthHandler;
	impl<T> AuthHandler<T> for NoneAuthHandler
	{
		fn jwt_secret<F : FnOnce() -> Option<T>>(&self, _state : &mut State, _decode_data : F) -> Option<Vec<u8>>
		{
			None
		}
	}
	
	#[test]
	fn test_auth_middleware_none_secret()
	{
		let middleware = <AuthMiddleware<TestData, NoneAuthHandler>>::from_source(AuthSource::AuthorizationHeader);
		State::with_new(|mut state| {
			let mut headers = HeaderMap::new();
			headers.insert(AUTHORIZATION, format!("Bearer {}", VALID_TOKEN).parse().unwrap());
			state.put(headers);
			middleware.auth_status(&mut state);
		});
	}
	
	#[derive(Default)]
	struct TestAssertingHandler;
	impl<T> AuthHandler<T> for TestAssertingHandler
	where T : Debug + Default + PartialEq
	{
		fn jwt_secret<F : FnOnce() -> Option<T>>(&self, _state : &mut State, decode_data : F) -> Option<Vec<u8>>
		{
			assert_eq!(decode_data(), Some(T::default()));
			Some(JWT_SECRET.to_vec())
		}
	}
	
	#[test]
	fn test_auth_middleware_decode_data()
	{
		let middleware = <AuthMiddleware<TestData, TestAssertingHandler>>::from_source(AuthSource::AuthorizationHeader);
		State::with_new(|mut state| {
			let mut headers = HeaderMap::new();
			headers.insert(AUTHORIZATION, format!("Bearer {}", VALID_TOKEN).parse().unwrap());
			state.put(headers);
			middleware.auth_status(&mut state);
		});
	}
	
	fn new_middleware<T>(source : AuthSource) -> AuthMiddleware<T, StaticAuthHandler>
	where T : DeserializeOwned + Send
	{
		AuthMiddleware::new(source, Default::default(), StaticAuthHandler::from_array(JWT_SECRET))
	}
	
	#[test]
	fn test_auth_middleware_no_token()
	{
		let middleware = new_middleware::<TestData>(AuthSource::AuthorizationHeader);
		State::with_new(|mut state| {
			let status = middleware.auth_status(&mut state);
			match status {
				AuthStatus::Unauthenticated => {},
				_ => panic!("Expected AuthStatus::Unauthenticated, got {:?}", status)
			};
		});
	}
	
	#[test]
	fn test_auth_middleware_expired_token()
	{
		let middleware = new_middleware::<TestData>(AuthSource::AuthorizationHeader);
		State::with_new(|mut state| {
			let mut headers = HeaderMap::new();
			headers.insert(AUTHORIZATION, format!("Bearer {}", EXPIRED_TOKEN).parse().unwrap());
			state.put(headers);
			let status = middleware.auth_status(&mut state);
			match status {
				AuthStatus::Expired => {},
				_ => panic!("Expected AuthStatus::Expired, got {:?}", status)
			};
		});
	}
	
	#[test]
	fn test_auth_middleware_invalid_token()
	{
		let middleware = new_middleware::<TestData>(AuthSource::AuthorizationHeader);
		State::with_new(|mut state| {
			let mut headers = HeaderMap::new();
			headers.insert(AUTHORIZATION, format!("Bearer {}", INVALID_TOKEN).parse().unwrap());
			state.put(headers);
			let status = middleware.auth_status(&mut state);
			match status {
				AuthStatus::Invalid => {},
				_ => panic!("Expected AuthStatus::Invalid, got {:?}", status)
			};
		});
	}
	
	#[test]
	fn test_auth_middleware_auth_header_token()
	{
		let middleware = new_middleware::<TestData>(AuthSource::AuthorizationHeader);
		State::with_new(|mut state| {
			let mut headers = HeaderMap::new();
			headers.insert(AUTHORIZATION, format!("Bearer {}", VALID_TOKEN).parse().unwrap());
			state.put(headers);
			let status = middleware.auth_status(&mut state);
			match status {
				AuthStatus::Authenticated(data) => assert_eq!(data, TestData::default()),
				_ => panic!("Expected AuthStatus::Authenticated, got {:?}", status)
			};
		})
	}
	
	#[test]
	fn test_auth_middleware_header_token()
	{
		let header_name = "x-znoiprwmvfexju";
		let middleware = new_middleware::<TestData>(AuthSource::Header(HeaderName::from_static(header_name)));
		State::with_new(|mut state| {
			let mut headers = HeaderMap::new();
			headers.insert(header_name, VALID_TOKEN.parse().unwrap());
			state.put(headers);
			let status = middleware.auth_status(&mut state);
			match status {
				AuthStatus::Authenticated(data) => assert_eq!(data, TestData::default()),
				_ => panic!("Expected AuthStatus::Authenticated, got {:?}", status)
			};
		})
	}
	
	#[test]
	fn test_auth_middleware_cookie_token()
	{
		let cookie_name = "znoiprwmvfexju";
		let middleware = new_middleware::<TestData>(AuthSource::Cookie(cookie_name.to_owned()));
		State::with_new(|mut state| {
			let mut jar = CookieJar::new();
			jar.add_original(Cookie::new(cookie_name, VALID_TOKEN));
			state.put(jar);
			let status = middleware.auth_status(&mut state);
			match status {
				AuthStatus::Authenticated(data) => assert_eq!(data, TestData::default()),
				_ => panic!("Expected AuthStatus::Authenticated, got {:?}", status)
			};
		})
	}
}
