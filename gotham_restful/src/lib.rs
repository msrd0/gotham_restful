#![allow(clippy::tabs_in_doc_comments)]
/*!
This crate is an extension to the popular [gotham web framework][gotham] for Rust. The idea is to
have several RESTful resources that can be added to the gotham router. This crate will take care
of everything else, like parsing path/query parameters, request bodies, and writing response
bodies, relying on [`serde`][serde] and [`serde_json`][serde_json] for (de)serializing. If you
enable the `openapi` feature, you can also generate an OpenAPI Specification from your RESTful
resources.

**Note:** The `stable` branch contains some bugfixes against the last release. The `master`
branch currently tracks gotham's master branch and the next release will use gotham 0.5.0 and be
compatible with the new future / async stuff.

# Usage

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
		route.resource::<UsersResource>("users");
	}));
}
```

Uploads and Downloads can also be handled:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::{router::builder::*, state::State};
# use gotham_restful::{DrawResources, Mime, Raw, Resource, Success};
# use serde::{Deserialize, Serialize};
#[derive(Resource)]
#[rest_resource(create)]
struct ImageResource;

#[derive(FromBody, RequestBody)]
#[supported_types(mime::IMAGE_GIF, mime::IMAGE_JPEG, mime::IMAGE_PNG)]
struct RawImage {
	content: Vec<u8>,
	content_type: Mime
}

#[rest_create(ImageResource)]
fn create(_state : &mut State, body : RawImage) -> Raw<Vec<u8>> {
	Raw::new(body.content, body.content_type)
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<ImageResource>("image");
# 	}));
# }
```

Look at the [example] for more methods and usage with the `openapi` feature.

# Known Issues

These are currently known major issues. For a complete list please see
[the issue tracker](https://gitlab.com/msrd0/gotham-restful/issues).
If you encounter any issues that aren't yet reported, please report them
[here](https://gitlab.com/msrd0/gotham-restful/issues/new).

 - Enabling the `openapi` feature might break code ([#4](https://gitlab.com/msrd0/gotham-restful/issues/4))
 - For `chrono`'s `DateTime` types, the format is `date-time` instead of `datetime` ([openapiv3#14](https://github.com/glademiller/openapiv3/pull/14))

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
#[macro_use] extern crate log;
#[macro_use] extern crate serde;

#[doc(no_inline)]
pub use gotham;
#[doc(no_inline)]
pub use gotham::{
	hyper::{header::HeaderName, StatusCode},
	state::{FromState, State}
};
#[doc(no_inline)]
pub use mime::Mime;

pub use gotham_restful_derive::*;

/// Not public API
#[doc(hidden)]
pub mod export
{
	pub use futures_util::future::FutureExt;
	
	pub use serde_json;
	
	#[cfg(feature = "database")]
	pub use gotham_middleware_diesel::Repo;
	
	#[cfg(feature = "openapi")]
	pub use indexmap::IndexMap;
	#[cfg(feature = "openapi")]
	pub use openapiv3 as openapi;
}

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::{
	AuthHandler,
	AuthMiddleware,
	AuthSource,
	AuthStatus,
	AuthValidation,
	StaticAuthHandler
};

pub mod matcher;

#[cfg(feature = "openapi")]
mod openapi;
#[cfg(feature = "openapi")]
pub use openapi::{
	builder::OpenapiInfo,
	router::GetOpenapi,
	types::{OpenapiSchema, OpenapiType}
};

mod resource;
pub use resource::{
	Resource,
	ResourceMethod,
	ResourceReadAll,
	ResourceRead,
	ResourceSearch,
	ResourceCreate,
	ResourceUpdateAll,
	ResourceUpdate,
	ResourceDeleteAll,
	ResourceDelete
};

mod response;
pub use response::Response;

mod result;
pub use result::{
	AuthError,
	AuthError::Forbidden,
	AuthErrorOrOther,
	AuthResult,
	AuthSuccess,
	IntoResponseError,
	NoContent,
	Raw,
	ResourceResult,
	Success
};

mod routing;
pub use routing::{DrawResources, DrawResourceRoutes};
#[cfg(feature = "openapi")]
pub use routing::WithOpenapi;

mod types;
pub use types::*;
