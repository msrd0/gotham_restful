use crate::{DrawResourceRoutes, ResourceResult};
use gotham::state::State;
use serde::de::DeserializeOwned;

pub trait Resource
{
	fn setup<D : DrawResourceRoutes>(route : D);
}

pub trait IndexResource<R : ResourceResult>
{
	fn index(state : &mut State) -> R;
}

pub trait GetResource<ID : DeserializeOwned>
{
	fn get(state : State, id : ID) -> dyn ResourceResult;
}

pub trait PostResource<Body : DeserializeOwned>
{
	fn post(state : State, body : Body) -> dyn ResourceResult;
}
