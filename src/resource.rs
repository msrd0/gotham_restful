use crate::ResourceResult;
use gotham::state::State;
use serde::{
	de::DeserializeOwned,
	ser::Serialize
};

pub trait Resource
{
	fn setup();
}

pub trait IndexResource<R : Serialize, E : Serialize, Res : ResourceResult<R, E>>
{
	fn index(state : &mut State) -> Res;
}

pub trait GetResource<ID : DeserializeOwned, R : Serialize, E : Serialize, Res : ResourceResult<R, E>>
{
	fn get(state : State, id : ID) -> Res;
}

pub trait PostResource<Body : DeserializeOwned, R : Serialize, E : Serialize, Res : ResourceResult<R, E>>
{
	fn post(state : State, body : Body) -> Res;
}
