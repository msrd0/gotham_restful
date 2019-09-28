use crate::{DrawResourceRoutes, ResourceResult};
use gotham::state::State;
use serde::de::DeserializeOwned;
use std::panic::RefUnwindSafe;

/// This trait must be implemented by every RESTful Resource. It will
/// allow you to register the different methods for this Resource.
pub trait Resource
{
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

/// Handle a POST request on the Resource root.
pub trait ResourceCreate<Body : DeserializeOwned, R : ResourceResult>
{
	fn create(state : &mut State, body : Body) -> R;
}

/// Handle a PUT request on the Resource root.
pub trait ResourceUpdateAll<Body : DeserializeOwned, R : ResourceResult>
{
	fn update_all(state : &mut State, body : Body) -> R;
}

/// Handle a PUT request on the Resource with an id.
pub trait ResourceUpdate<ID, Body : DeserializeOwned, R : ResourceResult>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	fn update(state : &mut State, id : ID, body : Body) -> R;
}
