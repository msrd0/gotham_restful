use super::{ResourceResult, into_response_helper};
use crate::{Response, ResponseBody};
#[cfg(feature = "openapi")]
use crate::OpenapiSchema;
use gotham::hyper::StatusCode;
use mime::{Mime, APPLICATION_JSON};
use std::{
	fmt::Debug,
	future::Future,
	pin::Pin,
	ops::{Deref, DerefMut}
};

/**
This can be returned from a resource when there is no cause of an error. It behaves similar to a
smart pointer like box, it that it implements `AsRef`, `Deref` and the likes.

Usage example:

```
# #[macro_use] extern crate gotham_restful_derive;
# mod doc_tests_are_broken {
# use gotham::state::State;
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
#
# #[derive(Resource)]
# struct MyResource;
#
#[derive(Deserialize, Serialize)]
# #[derive(OpenapiType)]
struct MyResponse {
	message: &'static str
}

#[rest_read_all(MyResource)]
fn read_all(_state: &mut State) -> Success<MyResponse> {
	let res = MyResponse { message: "I'm always happy" };
	res.into()
}
# }
```
*/
#[derive(Debug)]
pub struct Success<T>(T);

impl<T> AsMut<T> for Success<T>
{
	fn as_mut(&mut self) -> &mut T
	{
		&mut self.0
	}
}

impl<T> AsRef<T> for Success<T>
{
	fn as_ref(&self) -> &T
	{
		&self.0
	}
}

impl<T> Deref for Success<T>
{
	type Target = T;
	
	fn deref(&self) -> &T
	{
		&self.0
	}
}

impl<T> DerefMut for Success<T>
{
	fn deref_mut(&mut self) -> &mut T
	{
		&mut self.0
	}
}

impl<T> From<T> for Success<T>
{
	fn from(t : T) -> Self
	{
		Self(t)
	}
}

impl<T : Clone> Clone for Success<T>
{
	fn clone(&self) -> Self
	{
		Self(self.0.clone())
	}
}

impl<T : Copy> Copy for Success<T>
{
}

impl<T : Default> Default for Success<T>
{
	fn default() -> Self
	{
		Self(T::default())
	}
}

impl<T : ResponseBody> ResourceResult for Success<T>
where
	Self : Send
{
	type Err = serde_json::Error;
	
	fn into_response(self) -> Pin<Box<dyn Future<Output = Result<Response, Self::Err>> + Send>>
	{
		into_response_helper(|| Ok(Response::json(StatusCode::OK, serde_json::to_string(self.as_ref())?)))
	}
	
	fn accepted_types() -> Option<Vec<Mime>>
	{
		Some(vec![APPLICATION_JSON])
	}
	
	#[cfg(feature = "openapi")]
	fn schema() -> OpenapiSchema
	{
		T::schema()
	}
}


#[cfg(test)]
mod test
{
	use super::*;
	use crate::result::OrAllTypes;
	use futures_executor::block_on;
	
	#[derive(Debug, Default, Serialize)]
	#[cfg_attr(feature = "openapi", derive(crate::OpenapiType))]
	struct Msg
	{
		msg : String
	}
	
	#[test]
	fn success_always_successfull()
	{
		let success : Success<Msg> = Msg::default().into();
		let res = block_on(success.into_response()).expect("didn't expect error response");
		assert_eq!(res.status, StatusCode::OK);
		assert_eq!(res.mime, Some(APPLICATION_JSON));
		assert_eq!(res.full_body().unwrap(), r#"{"msg":""}"#.as_bytes());
	}
	
	#[test]
	fn success_accepts_json()
	{
		assert!(<Success<Msg>>::accepted_types().or_all_types().contains(&APPLICATION_JSON))
	}
}
