use crate::{DrawResourceRoutes, RequestBody, ResourceResult, ResourceType};
use gotham::{
	extractor::QueryStringExtractor,
	hyper::Body,
	state::State
};
use serde::de::DeserializeOwned;
use std::{
	future::Future,
	panic::RefUnwindSafe,
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
	type ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static;
	
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
pub trait ResourceUpdateAll : ResourceMethod
{
	type Body : RequestBody;
	
	fn update_all(state : State, body : Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a PUT request on the Resource with an id.
pub trait ResourceUpdate : ResourceMethod
{
	type Body : RequestBody;
	type ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static;
	
	fn update(state : State, id : Self::ID, body : Self::Body) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a DELETE request on the Resource root.
pub trait ResourceDeleteAll : ResourceMethod
{
	fn delete_all(state : State) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}

/// Handle a DELETE request on the Resource with an id.
pub trait ResourceDelete : ResourceMethod
{
	type ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static;
	
	fn delete(state : State, id : Self::ID) -> Pin<Box<dyn Future<Output = (State, Self::Res)> + Send>>;
}
