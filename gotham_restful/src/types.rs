#[cfg(feature = "openapi")]
use crate::OpenapiType;

use serde::{de::DeserializeOwned, Serialize};

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
