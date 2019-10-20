/*!
This crate is an extension to the popular [gotham web framework][gotham] for Rust. The idea is to
have several RESTful resources that can be added to the gotham router. This crate will take care
of everything else, like parsing path/query parameters, request bodies, and writing response
bodies, relying on [`serde`][serde] and [`serde_json`][serde_json] for (de)serializing. If you
enable the `openapi` feature, you can also generate an OpenAPI Specification from your RESTful
resources.

# Usage

To use this crate, add the following to your `Cargo.toml`:

```toml
[dependencies]
gotham_restful = "0.0.1"
```

A basic server with only one resource, handling a simple `GET` request, could look like this:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::{router::builder::*, state::State};
# use gotham_restful::{DrawResources, Resource, Success};
# use serde::{Deserialize, Serialize};
/// Our RESTful Resource.
#[derive(Resource)]
#[rest_resource(read_all)]
struct UsersResource;

/// Our return type.
#[derive(Deserialize, Serialize)]
# #[derive(OpenapiType)]
struct User {
	id: i64,
	username: String,
	email: String
}

/// Our handler method.
#[rest_read_all(UsersResource)]
fn read_all(_state: &mut State) -> Success<Vec<User>> {
	vec![User {
		id: 1,
		username: "h4ck3r".to_string(),
		email: "h4ck3r@example.org".to_string()
	}].into()
}

/// Our main method.
fn main() {
	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
		route.resource::<UsersResource, _>("users");
	}));
}
```

Uploads and Downloads can also be handled, but you need to specify the mime type manually:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::{router::builder::*, state::State};
# use gotham_restful::{DrawResources, Raw, Resource, Success};
# use serde::{Deserialize, Serialize};
#[derive(Resource)]
#[rest_resource(create)]
struct ImageResource;

#[derive(FromBody, RequestBody)]
#[supported_types(mime::IMAGE_GIF, mime::IMAGE_JPEG, mime::IMAGE_PNG)]
struct RawImage(Vec<u8>);

#[rest_create(ImageResource)]
fn create(_state : &mut State, body : RawImage) -> Raw<Vec<u8>> {
	Raw::new(body.0, mime::APPLICATION_OCTET_STREAM)
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<ImageResource, _>("image");
# 	}));
# }
```

Look at the [example] for more methods and usage with the `openapi` feature.

# License

Licensed under your option of:
 - [Apache License Version 2.0](https://gitlab.com/msrd0/gotham-restful/blob/master/LICENSE-Apache)
 - [Eclipse Public License Version 2.0](https://gitlab.com/msrd0/gotham-restful/blob/master/LICENSE-EPL)


[example]: https://gitlab.com/msrd0/gotham-restful/tree/master/example
[gotham]: https://gotham.rs/
[serde]: https://github.com/serde-rs/serde#serde-----
[serde_json]: https://github.com/serde-rs/json#serde-json----
*/

// weird proc macro issue
extern crate self as gotham_restful;

#[macro_use] extern crate gotham_derive;
#[macro_use] extern crate serde;

#[doc(no_inline)]
pub use hyper::{Chunk, StatusCode};
#[doc(no_inline)]
pub use mime::Mime;

pub use gotham_restful_derive::*;

/// Not public API
#[doc(hidden)]
pub mod export
{
	#[cfg(feature = "openapi")]
	pub use indexmap::IndexMap;
	#[cfg(feature = "openapi")]
	pub use openapiv3 as openapi;
}

#[cfg(feature = "openapi")]
mod openapi;
#[cfg(feature = "openapi")]
pub use openapi::{
	router::{GetOpenapi, OpenapiRouter},
	types::{OpenapiSchema, OpenapiType}
};

mod resource;
pub use resource::{
	Resource,
	ResourceReadAll,
	ResourceRead,
	ResourceSearch,
	ResourceCreate,
	ResourceUpdateAll,
	ResourceUpdate,
	ResourceDeleteAll,
	ResourceDelete
};

mod result;
pub use result::{
	NoContent,
	Raw,
	ResourceResult,
	Response,
	Success
};

mod routing;
pub use routing::{DrawResources, DrawResourceRoutes};
#[cfg(feature = "openapi")]
pub use routing::WithOpenapi;

mod types;
pub use types::*;
