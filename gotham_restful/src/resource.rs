use crate::{DrawResourceRoutes, RequestBody, ResourceResult, ResourceType};
use gotham::{
	extractor::QueryStringExtractor,
	state::State
};
use hyper::Body;
use serde::de::DeserializeOwned;
use std::panic::RefUnwindSafe;

/// This trait must be implemented by every RESTful Resource. It will
/// allow you to register the different methods for this Resource.
pub trait Resource
{
	/// The name of this resource. Must be unique.
	fn name() -> String;

	/// Setup all routes of this resource. Take a look at the rest_resource!
	/// macro if you don't feel like caring yourself.
	fn setup<D : DrawResourceRoutes>(route : D);
}

pub trait ResourceMethod
{
	type Res : ResourceResult;
}

/// Handle a GET request on the Resource root.
pub trait ResourceReadAll : ResourceMethod
{
	fn read_all(state : &mut State) -> Self::Res;
}

/// Handle a GET request on the Resource with an id.
pub trait ResourceRead : ResourceMethod
{
	type ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static;
	
	fn read(state : &mut State, id : Self::ID) -> Self::Res;
}

/// Handle a GET request on the Resource with additional search parameters.
pub trait ResourceSearch : ResourceMethod
{
	type Query : ResourceType + QueryStringExtractor<Body> + Sync;
	
	fn search(state : &mut State, query : Self::Query) -> Self::Res;
}

/// Handle a POST request on the Resource root.
pub trait ResourceCreate : ResourceMethod
{
	type Body : RequestBody;
	
	fn create(state : &mut State, body : Self::Body) -> Self::Res;
}

/// Handle a PUT request on the Resource root.
pub trait ResourceUpdateAll : ResourceMethod
{
	type Body : RequestBody;
	
	fn update_all(state : &mut State, body : Self::Body) -> Self::Res;
}

/// Handle a PUT request on the Resource with an id.
pub trait ResourceUpdate : ResourceMethod
{
	type Body : RequestBody;
	type ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static;
	
	fn update(state : &mut State, id : Self::ID, body : Self::Body) -> Self::Res;
}

/// Handle a DELETE request on the Resource root.
pub trait ResourceDeleteAll : ResourceMethod
{
	fn delete_all(state : &mut State) -> Self::Res;
}

/// Handle a DELETE request on the Resource with an id.
pub trait ResourceDelete : ResourceMethod
{
	type ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static;
	
	fn delete(state : &mut State, id : Self::ID) -> Self::Res;
}
