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
pub trait IndexResource<R : ResourceResult>
{
	fn index(state : &mut State) -> R;
}

/// Handle a GET request on the Resource with an id.
pub trait GetResource<ID, R : ResourceResult>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	fn get(state : &mut State, id : ID) -> R;
}

/// Handle a POST request on the Resource root.
pub trait CreateResource<Body : DeserializeOwned, R : ResourceResult>
{
	fn create(state : &mut State, body : Body) -> R;
}

/// Handle a PUT request on the Resource root.
pub trait ChangeAllResource<Body : DeserializeOwned, R : ResourceResult>
{
	fn change_all(state : &mut State, body : Body) -> R;
}

/// Handle a PUT request on the Resource with an id.
pub trait ChangeResource<ID, Body : DeserializeOwned, R : ResourceResult>
where
	ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static
{
	fn change(state : &mut State, id : ID, body : Body) -> R;
}
