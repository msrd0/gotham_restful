#[cfg(feature = "openapi")]
use crate::OpenapiType;

use gotham::hyper::body::Bytes;
use mime::{Mime, APPLICATION_JSON};
use serde::{de::DeserializeOwned, Serialize};
use std::{
	error::Error,
	panic::RefUnwindSafe
};
use thiserror::Error;

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


/**
This trait should be implemented for every type that can be built from an HTTP request body
plus its media type. For most use cases it is sufficient to derive this trait, you usually
don't need to manually implement this. Therefore, make sure that the first variable of
your struct can be built from [`Bytes`], and the second one can be build from [`Mime`].
If you have any additional variables, they need to be `Default`. This is an example of
such a struct:

```rust
# #[macro_use] extern crate gotham_restful;
# use gotham_restful::*;
#[derive(FromBody, RequestBody)]
#[supported_types(mime::IMAGE_GIF, mime::IMAGE_JPEG, mime::IMAGE_PNG)]
struct RawImage {
	content: Vec<u8>,
	content_type: Mime
}
```

 [`Bytes`]: ../bytes/struct.Bytes.html
 [`Mime`]: ../mime/struct.Mime.html
*/
pub trait FromBody : Sized
{
	/// The error type returned by the conversion if it was unsuccessfull. When using the derive
	/// macro, there is no way to trigger an error, so
	/// [`FromBodyNoError`](struct.FromBodyNoError.html) is used here.
	type Err : Error;
	
	/// Perform the conversion.
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

/// This error type can be used by [`FromBody`](trait.FromBody.html) implementations when there
/// is no need to return any errors.
#[derive(Clone, Copy, Debug, Error)]
#[error("No Error")]
pub struct FromBodyNoError;


/**
A type that can be used inside a request body. Implemented for every type that is deserializable
with serde. If the `openapi` feature is used, it must also be of type [`OpenapiType`].

If you want a non-deserializable type to be used as a request body, e.g. because you'd like to
get the raw data, you can derive it for your own type. All you need is to have a type implementing
[`FromBody`] and optionally a list of supported media types:

```rust
# #[macro_use] extern crate gotham_restful;
# use gotham_restful::*;
#[derive(FromBody, RequestBody)]
#[supported_types(mime::IMAGE_GIF, mime::IMAGE_JPEG, mime::IMAGE_PNG)]
struct RawImage {
	content: Vec<u8>,
	content_type: Mime
}
```

 [`FromBody`]: trait.FromBody.html
 [`OpenapiType`]: trait.OpenapiType.html
*/
pub trait RequestBody : ResourceType + FromBody
{
	/// Return all types that are supported as content types. Use `None` if all types are supported.
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
