#[cfg(feature = "openapi")]
use crate::OpenapiType;
use crate::result::ResourceError;

use gotham::hyper::body::Bytes;
use mime::{Mime, APPLICATION_JSON};
use serde::{de::DeserializeOwned, Serialize};
use std::panic::RefUnwindSafe;

#[cfg(not(feature = "openapi"))]
pub trait ResourceType
{
}

#[cfg(not(feature = "openapi"))]
impl<T> ResourceType for T
{
}

#[cfg(feature = "openapi")]
pub trait ResourceType : OpenapiType
{
}

#[cfg(feature = "openapi")]
impl<T : OpenapiType> ResourceType for T
{
}


/// A type that can be used inside a response body. Implemented for every type that is
/// serializable with serde. If the `openapi` feature is used, it must also be of type
/// `OpenapiType`.
pub trait ResponseBody : ResourceType + Serialize
{
}

impl<T : ResourceType + Serialize> ResponseBody for T
{
}


/// This trait must be implemented by every type that can be used as a request body. It allows
/// to create the type from a hyper body chunk and it's content type.
pub trait FromBody : Sized
{
	type Err : Into<ResourceError>;
	
	/// Create the request body from a raw body and the content type.
	fn from_body(body : Bytes, content_type : Mime) -> Result<Self, Self::Err>;
}

impl<T : DeserializeOwned> FromBody for T
{
	type Err = serde_json::Error;
	
	fn from_body(body : Bytes, _content_type : Mime) -> Result<Self, Self::Err>
	{
		serde_json::from_slice(&body)
	}
}

/// A type that can be used inside a request body. Implemented for every type that is
/// deserializable with serde. If the `openapi` feature is used, it must also be of type
/// `OpenapiType`.
pub trait RequestBody : ResourceType + FromBody
{
	/// Return all types that are supported as content types.
	fn supported_types() -> Option<Vec<Mime>>
	{
		None
	}
}

impl<T : ResourceType + DeserializeOwned> RequestBody for T
{
	fn supported_types() -> Option<Vec<Mime>>
	{
		Some(vec![APPLICATION_JSON])
	}
}

/// A type than can be used as a parameter to a resource method. Implemented for every type
/// that is deserialize and thread-safe. If the `openapi` feature is used, it must also be of
/// type `OpenapiType`.
pub trait ResourceID : ResourceType + DeserializeOwned + Clone + RefUnwindSafe + Send + Sync
{
}

impl<T : ResourceType + DeserializeOwned + Clone + RefUnwindSafe + Send + Sync> ResourceID for T
{
}
