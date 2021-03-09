use super::IntoResponse;
#[cfg(feature = "openapi")]
use crate::ResponseSchema;
use crate::{Response, ResponseBody};
use futures_util::future::{self, FutureExt};
use gotham::hyper::{
	header::{HeaderMap, HeaderValue, IntoHeaderName},
	StatusCode
};
use mime::{Mime, APPLICATION_JSON};
#[cfg(feature = "openapi")]
use openapi_type::OpenapiSchema;
use std::{fmt::Debug, future::Future, pin::Pin};

/**
This can be returned from a resource when there is no cause of an error.

Usage example:

```
# #[macro_use] extern crate gotham_restful_derive;
# mod doc_tests_are_broken {
# use gotham::state::State;
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
#
# #[derive(Resource)]
# #[resource(read_all)]
# struct MyResource;
#
#[derive(Deserialize, Serialize)]
# #[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
struct MyResponse {
	message: &'static str
}

#[read_all]
fn read_all() -> Success<MyResponse> {
	let res = MyResponse { message: "I'm always happy" };
	res.into()
}
# }
```
*/
#[derive(Clone, Debug, Default)]
pub struct Success<T> {
	value: T,
	headers: HeaderMap
}

impl<T> From<T> for Success<T> {
	fn from(t: T) -> Self {
		Self {
			value: t,
			headers: HeaderMap::new()
		}
	}
}

impl<T> Success<T> {
	/// Set a custom HTTP header. If a header with this name was set before, its value is being updated.
	pub fn header<K: IntoHeaderName>(&mut self, name: K, value: HeaderValue) {
		self.headers.insert(name, value);
	}

	/// Allow manipulating HTTP headers.
	pub fn headers_mut(&mut self) -> &mut HeaderMap {
		&mut self.headers
	}
}

impl<T: ResponseBody> IntoResponse for Success<T> {
	type Err = serde_json::Error;

	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>> {
		let res =
			serde_json::to_string(&self.value).map(|body| Response::json(StatusCode::OK, body).with_headers(self.headers));
		future::ready(res).boxed()
	}

	fn accepted_types() -> Option<Vec<Mime>> {
		Some(vec![APPLICATION_JSON])
	}
}

#[cfg(feature = "openapi")]
impl<T: ResponseBody> ResponseSchema for Success<T> {
	fn schema() -> OpenapiSchema {
		T::schema()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::response::OrAllTypes;
	use futures_executor::block_on;
	use gotham::hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN;

	#[derive(Debug, Default, Serialize)]
	#[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
	struct Msg {
		msg: String
	}

	#[test]
	fn success_always_successfull() {
		let success: Success<Msg> = Msg::default().into();
		let res = block_on(success.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body().unwrap(), br#"{"msg":""}"#);
		#[cfg(feature = "openapi")]
		assert_eq!(<Success<Msg>>::default_status(), StatusCode::OK);
	}

	#[test]
	fn success_custom_headers() {
		let mut success: Success<Msg> = Msg::default().into();
		success.header(ACCESS_CONTROL_ALLOW_ORIGIN, HeaderValue::from_static("*"));
		let res = block_on(success.into_response()).expect("didn't expect error response");
		let cors = res.headers.get(ACCESS_CONTROL_ALLOW_ORIGIN);
		assert_eq!(cors.map(|value| value.to_str().unwrap()), Some("*"));
	}

	#[test]
	fn success_accepts_json() {
		assert!(<Success<Msg>>::accepted_types().or_all_types().contains(&APPLICATION_JSON))
	}
}
