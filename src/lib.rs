#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate serde;

pub use hyper::StatusCode;
use serde::{de::DeserializeOwned, Serialize};

pub mod helper;

#[cfg(feature = "openapi")]
pub mod openapi;
#[cfg(feature = "openapi")]
pub use openapi::{
	router::{GetOpenapi, OpenapiRouter},
	types::OpenapiType
};

mod resource;
pub use resource::{
	Resource,
	ResourceReadAll,
	ResourceRead,
	ResourceCreate,
	ResourceUpdateAll,
	ResourceUpdate,
	ResourceDeleteAll,
	ResourceDelete
};

mod result;
pub use result::{ResourceResult, Success};

mod routing;
pub use routing::{DrawResources, DrawResourceRoutes};
#[cfg(feature = "openapi")]
pub use routing::WithOpenapi;


/// A type that can be used inside a request or response body. Implemented for every type
/// that is serializable with serde, however, it is recommended to use the rest_struct!
/// macro to create one.
#[cfg(not(feature = "openapi"))]
pub trait ResourceType : DeserializeOwned + Serialize
{
}

#[cfg(not(feature = "openapi"))]
impl<T : DeserializeOwned + Serialize> ResourceType for T
{
}

/// A type that can be used inside a request or response body. Implemented for every type
/// that is serializable with serde, however, it is recommended to use the rest_struct!
/// macro to create one.
#[cfg(feature = "openapi")]
pub trait ResourceType : OpenapiType + DeserializeOwned + Serialize
{
}

#[cfg(feature = "openapi")]
impl<T : OpenapiType + DeserializeOwned + Serialize> ResourceType for T
{
}
