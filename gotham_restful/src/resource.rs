use crate::{DrawResourceRoutes, RequestBody, ResourceID, ResourceResult, ResourceType};
use gotham::{
	extractor::QueryStringExtractor,
	hyper::Body,
	state::State
};
use std::{
	future::Future,
	pin::Pin
};

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
	type Res : ResourceResult + Send + 'static;
	
	#[cfg(feature = "openapi")]
	fn operation_id() -> Option<String>
	{
		None
	}
	
	fn wants_auth() -> bool
	{
		false
	}
}

/// Handle a GET request on the Resource root.
pub trait ResourceReadAll : ResourceMethod
{
	fn read_all(state : State) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a GET request on the Resource with an id.
pub trait ResourceRead : ResourceMethod
{
	type ID : ResourceID + 'static;
	
	fn read(state : State, id : Self::ID) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a GET request on the Resource with additional search parameters.
pub trait ResourceSearch : ResourceMethod
{
	type Query : ResourceType + QueryStringExtractor<Body> + Sync;
	
	fn search(state : State, query : Self::Query) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a POST request on the Resource root.
pub trait ResourceCreate : ResourceMethod
{
	type Body : RequestBody;
	
	fn create(state : State, body : Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a PUT request on the Resource root.
pub trait ResourceChangeAll : ResourceMethod
{
	type Body : RequestBody;
	
	fn change_all(state : State, body : Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a PUT request on the Resource with an id.
pub trait ResourceChange : ResourceMethod
{
	type Body : RequestBody;
	type ID : ResourceID + 'static;
	
	fn change(state : State, id : Self::ID, body : Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a DELETE request on the Resource root.
pub trait ResourceRemoveAll : ResourceMethod
{
	fn remove_all(state : State) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a DELETE request on the Resource with an id.
pub trait ResourceRemove : ResourceMethod
{
	type ID : ResourceID + 'static;
	
	fn remove(state : State, id : Self::ID) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}
