use crate::{DrawResourceRoutes, ResourceResult, ResourceType};
use gotham::{
	router::response::extender::StaticResponseExtender,
	state::{State, StateData}
};
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

/// Handle a GET request on the Resource root.
pub trait ResourceReadAll<R : ResourceResult>
{
	fn read_all(state : &mut State) -> R;
}

/// Handle a GET request on the Resource with an id.
pub trait ResourceRead<ID, R : ResourceResult>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	fn read(state : &mut State, id : ID) -> R;
}

/// Handle a GET request on the Resource with additional search parameters.
pub trait ResourceSearch<Query : ResourceType, R : ResourceResult>
where
	Query : ResourceType + StateData + StaticResponseExtender
{
	fn search(state : &mut State, query : Query) -> R;
}

/// Handle a POST request on the Resource root.
pub trait ResourceCreate<Body : ResourceType, R : ResourceResult>
{
	fn create(state : &mut State, body : Body) -> R;
}

/// Handle a PUT request on the Resource root.
pub trait ResourceUpdateAll<Body : ResourceType, R : ResourceResult>
{
	fn update_all(state : &mut State, body : Body) -> R;
}

/// Handle a PUT request on the Resource with an id.
pub trait ResourceUpdate<ID, Body : ResourceType, R : ResourceResult>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	fn update(state : &mut State, id : ID, body : Body) -> R;
}

/// Handle a DELETE request on the Resource root.
pub trait ResourceDeleteAll<R : ResourceResult>
{
	fn delete_all(state : &mut State) -> R;
}

/// Handle a DELETE request on the Resource with an id.
pub trait ResourceDelete<ID, R : ResourceResult>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	fn delete(state : &mut State, id : ID) -> R;
}
