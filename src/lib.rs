#![warn(missing_debug_implementations, rust_2018_idioms, unreachable_pub)]
#![deny(warnings)]
#![forbid(unsafe_code)]
// can we have a lint for spaces in doc comments please?
#![cfg_attr(feature = "cargo-clippy", allow(clippy::tabs_in_doc_comments))]
// intra-doc links only fully work when OpenAPI is enabled
#![cfg_attr(feature = "openapi", deny(broken_intra_doc_links))]
#![cfg_attr(not(feature = "openapi"), allow(broken_intra_doc_links))]
/*!
This crate is an extension to the popular [gotham web framework][gotham] for Rust. It allows you to
create resources with assigned endpoints that aim to be a more convenient way of creating handlers
for requests.

# Features

 - Automatically parse **JSON** request and produce response bodies
 - Allow using **raw** request and response bodies
 - Convenient **macros** to create responses that can be registered with gotham's router
 - Auto-Generate an **OpenAPI** specification for your API
 - Manage **CORS** headers so you don't have to
 - Manage **Authentication** with JWT
 - Integrate diesel connection pools for easy **database** integration

# Safety

This crate is just as safe as you'd expect from anything written in safe Rust - and
`#![forbid(unsafe_code)]` ensures that no unsafe was used.

# Endpoints

There are a set of pre-defined endpoints that should cover the majority of REST APIs. However,
it is also possible to define your own endpoints.

## Pre-defined Endpoints

Assuming you assign `/foobar` to your resource, the following pre-defined endpoints exist:

| Endpoint Name | Required Arguments | HTTP Verb | HTTP Path      |
| ------------- | ------------------ | --------- | -------------- |
| read_all      |                    | GET       | /foobar        |
| read          | id                 | GET       | /foobar/:id    |
| search        | query              | GET       | /foobar/search |
| create        | body               | POST      | /foobar        |
| update_all    | body               | PUT       | /foobar        |
| update        | id, body           | PUT       | /foobar/:id    |
| delete_all    |                    | DELETE    | /foobar        |
| delete        | id                 | DELETE    | /foobar/:id    |

Each of those endpoints has a macro that creates the neccessary boilerplate for the Resource. A
simple example looks like this:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::router::builder::*;
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
/// Our RESTful resource.
#[derive(Resource)]
#[resource(read)]
struct FooResource;

/// The return type of the foo read endpoint.
#[derive(Serialize)]
# #[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
struct Foo {
	id: u64
}

/// The foo read endpoint.
#[read]
fn read(id: u64) -> Success<Foo> {
	Foo { id }.into()
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<FooResource>("foo");
# 	}));
# }
```

## Custom Endpoints

Defining custom endpoints is done with the `#[endpoint]` macro. The syntax is similar to that
of the pre-defined endpoints, but you need to give it more context:

```rust,no_run
# #[macro_use] extern crate gotham_derive;
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::router::builder::*;
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
use gotham_restful::gotham::hyper::Method;

#[derive(Resource)]
#[resource(custom_endpoint)]
struct CustomResource;

/// This type is used to parse path parameters.
#[derive(Clone, Deserialize, StateData, StaticResponseExtender)]
# #[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
struct CustomPath {
	name: String
}

#[endpoint(uri = "custom/:name/read", method = "Method::GET", params = false, body = false)]
fn custom_endpoint(path: CustomPath) -> Success<String> {
	path.name.into()
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<CustomResource>("custom");
# 	}));
# }
```

# Arguments

Some endpoints require arguments. Those should be
 * **id** Should be a deserializable json-primitive like [`i64`] or [`String`].
 * **body** Should be any deserializable object, or any type implementing [`RequestBody`].
 * **query** Should be any deserializable object whose variables are json-primitives. It will
   however not be parsed from json, but from HTTP GET parameters like in `search?id=1`. The
   type needs to implement [`QueryStringExtractor`](gotham::extractor::QueryStringExtractor).

Additionally, all handlers may take a reference to gotham's [`State`]. Please note that for async
handlers, it needs to be a mutable reference until rustc's lifetime checks across await bounds
improve.

# Uploads and Downloads

By default, every request body is parsed from json, and every respone is converted to json using
[serde_json]. However, you may also use raw bodies. This is an example where the request body
is simply returned as the response again, no json parsing involved:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::router::builder::*;
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
#[derive(Resource)]
#[resource(create)]
struct ImageResource;

#[derive(FromBody, RequestBody)]
#[supported_types(mime::IMAGE_GIF, mime::IMAGE_JPEG, mime::IMAGE_PNG)]
struct RawImage {
	content: Vec<u8>,
	content_type: Mime
}

#[create]
fn create(body : RawImage) -> Raw<Vec<u8>> {
	Raw::new(body.content, body.content_type)
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<ImageResource>("image");
# 	}));
# }
```

# Custom HTTP Headers

You can read request headers from the state as you would in any other gotham handler, and specify
custom response headers using [Response::header].

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# use gotham::hyper::header::{ACCEPT, HeaderMap, VARY};
# use gotham::{router::builder::*, state::State};
# use gotham_restful::*;
#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all]
async fn read_all(state: &mut State) -> NoContent {
	let headers: &HeaderMap = state.borrow();
	let accept = &headers[ACCEPT];
# drop(accept);

	let mut res = NoContent::default();
	res.header(VARY, "accept".parse().unwrap());
	res
}
# fn main() {
# 	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
# 		route.resource::<FooResource>("foo");
# 	}));
# }
```

# Features

To make life easier for common use-cases, this create offers a few features that might be helpful
when you implement your web server.  The complete feature list is
 - [`auth`](#authentication-feature) Advanced JWT middleware
 - `chrono` openapi support for chrono types
 - `full` enables all features except `without-openapi`
 - [`cors`](#cors-feature) CORS handling for all endpoint handlers
 - [`database`](#database-feature) diesel middleware support
 - `errorlog` log errors returned from endpoint handlers
 - [`openapi`](#openapi-feature) router additions to generate an openapi spec
 - `uuid` openapi support for uuid
 - `without-openapi` (**default**) disables `openapi` support.

## Authentication Feature

In order to enable authentication support, enable the `auth` feature gate. This allows you to
register a middleware that can automatically check for the existence of an JWT authentication
token. Besides being supported by the endpoint macros, it supports to lookup the required JWT secret
with the JWT data, hence you can use several JWT secrets and decide on the fly which secret to use.
None of this is currently supported by gotham's own JWT middleware.

A simple example that uses only a single secret looks like this:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# #[cfg(feature = "auth")]
# mod auth_feature_enabled {
# use gotham::{router::builder::*, pipeline::{new_pipeline, single::single_pipeline}, state::State};
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
#[derive(Resource)]
#[resource(read)]
struct SecretResource;

#[derive(Serialize)]
# #[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
struct Secret {
	id: u64,
	intended_for: String
}

#[derive(Deserialize, Clone)]
struct AuthData {
	sub: String,
	exp: u64
}

#[read]
fn read(auth: AuthStatus<AuthData>, id: u64) -> AuthSuccess<Secret> {
	let intended_for = auth.ok()?.sub;
	Ok(Secret { id, intended_for })
}

fn main() {
	let auth: AuthMiddleware<AuthData, _> = AuthMiddleware::new(
		AuthSource::AuthorizationHeader,
		AuthValidation::default(),
		StaticAuthHandler::from_array(b"zlBsA2QXnkmpe0QTh8uCvtAEa4j33YAc")
	);
	let (chain, pipelines) = single_pipeline(new_pipeline().add(auth).build());
	gotham::start("127.0.0.1:8080", build_router(chain, pipelines, |route| {
		route.resource::<SecretResource>("secret");
	}));
}
# }
```

## CORS Feature

The cors feature allows an easy usage of this web server from other origins. By default, only
the `Access-Control-Allow-Methods` header is touched. To change the behaviour, add your desired
configuration as a middleware.

A simple example that allows authentication from every origin (note that `*` always disallows
authentication), and every content type, looks like this:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# #[cfg(feature = "cors")]
# mod cors_feature_enabled {
# use gotham::{hyper::header::*, router::builder::*, pipeline::{new_pipeline, single::single_pipeline}, state::State};
# use gotham_restful::{*, cors::*};
# use serde::{Deserialize, Serialize};
#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all]
fn read_all() {
	// your handler
}

fn main() {
	let cors = CorsConfig {
		origin: Origin::Copy,
		headers: Headers::List(vec![CONTENT_TYPE]),
		max_age: 0,
		credentials: true
	};
	let (chain, pipelines) = single_pipeline(new_pipeline().add(cors).build());
	gotham::start("127.0.0.1:8080", build_router(chain, pipelines, |route| {
		route.resource::<FooResource>("foo");
	}));
}
# }
```

The cors feature can also be used for non-resource handlers. Take a look at [`CorsRoute`]
for an example.

## Database Feature

The database feature allows an easy integration of [diesel] into your handler functions. Please
note however that due to the way gotham's diesel middleware implementation, it is not possible
to run async code while holding a database connection. If you need to combine async and database,
you'll need to borrow the connection from the [`State`] yourself and return a boxed future.

A simple non-async example looks like this:

```rust,no_run
# #[macro_use] extern crate diesel;
# #[macro_use] extern crate gotham_restful_derive;
# #[cfg(feature = "database")]
# mod database_feature_enabled {
# use diesel::{table, PgConnection, QueryResult, RunQueryDsl};
# use gotham::{router::builder::*, pipeline::{new_pipeline, single::single_pipeline}, state::State};
# use gotham_middleware_diesel::DieselMiddleware;
# use gotham_restful::*;
# use serde::{Deserialize, Serialize};
# use std::env;
# table! {
#   foo (id) {
#     id -> Int8,
#     value -> Text,
#   }
# }
#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[derive(Queryable, Serialize)]
# #[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
struct Foo {
	id: i64,
	value: String
}

#[read_all]
fn read_all(conn: &PgConnection) -> QueryResult<Vec<Foo>> {
	foo::table.load(conn)
}

type Repo = gotham_middleware_diesel::Repo<PgConnection>;

fn main() {
	let repo = Repo::new(&env::var("DATABASE_URL").unwrap());
	let diesel = DieselMiddleware::new(repo);

	let (chain, pipelines) = single_pipeline(new_pipeline().add(diesel).build());
	gotham::start("127.0.0.1:8080", build_router(chain, pipelines, |route| {
		route.resource::<FooResource>("foo");
	}));
}
# }
```

## OpenAPI Feature

The OpenAPI feature is probably the most powerful one of this crate. Definitely read this section
carefully both as a binary as well as a library author to avoid unwanted suprises.

In order to automatically create an openapi specification, gotham-restful needs knowledge over
all routes and the types returned. `serde` does a great job at serialization but doesn't give
enough type information, so all types used in the router need to implement
[`OpenapiType`](openapi_type::OpenapiType). This can be derived for almoust any type and there
should be no need to implement it manually. A simple example looks like this:

```rust,no_run
# #[macro_use] extern crate gotham_restful_derive;
# #[cfg(feature = "openapi")]
# mod openapi_feature_enabled {
# use gotham::{router::builder::*, state::State};
# use gotham_restful::*;
# use openapi_type::OpenapiType;
# use serde::{Deserialize, Serialize};
#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[derive(OpenapiType, Serialize)]
struct Foo {
	bar: String
}

#[read_all]
fn read_all() -> Success<Foo> {
	Foo { bar: "Hello World".to_owned() }.into()
}

fn main() {
	gotham::start("127.0.0.1:8080", build_simple_router(|route| {
		let info = OpenapiInfo {
			title: "My Foo API".to_owned(),
			version: "0.1.0".to_owned(),
			urls: vec!["https://example.org/foo/api/v1".to_owned()]
		};
		route.with_openapi(info, |mut route| {
			route.resource::<FooResource>("foo");
			route.openapi_spec("openapi");
			route.openapi_doc("/");
		});
	}));
}
# }
```

Above example adds the resource as before, but adds two other endpoints as well: `/openapi` and `/`.
The first one will return the generated openapi specification in JSON format, allowing you to easily
generate clients in different languages without worying to exactly replicate your api in each of those
languages. The second one will return documentation in HTML format, so you can easily view your
api and share it with other people.

However, please note that by default, the `without-openapi` feature of this crate is enabled.
Disabling it in favour of the `openapi` feature will add additional type bounds and method requirements
to some of the traits and types in this crate, for example instead of [`Endpoint`] you now have to
implement [`EndpointWithSchema`]. This means that some code might only compile on either feature, but not
on both. If you are writing a library that uses gotham-restful, it is strongly recommended to pass both
features through and conditionally enable the openapi code, like this:

```rust
# #[macro_use] extern crate gotham_restful;
# use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(openapi_type::OpenapiType))]
struct Foo;
```

 [diesel]: https://diesel.rs/
 [`State`]: gotham::state::State
*/

#[cfg(all(feature = "openapi", feature = "without-openapi"))]
compile_error!("The 'openapi' and 'without-openapi' features cannot be combined");

#[cfg(all(not(feature = "openapi"), not(feature = "without-openapi")))]
compile_error!("Either the 'openapi' or 'without-openapi' feature needs to be enabled");

// weird proc macro issue
extern crate self as gotham_restful;

#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate gotham_restful_derive;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde;

#[cfg(test)]
#[macro_use]
extern crate pretty_assertions;

#[doc(no_inline)]
pub use gotham;
#[doc(no_inline)]
pub use mime::Mime;

pub use gotham_restful_derive::*;

/// Not public API
#[doc(hidden)]
pub mod private {
	pub use crate::routing::PathExtractor as IdPlaceholder;

	pub use futures_util::future::{BoxFuture, FutureExt};

	pub use serde_json;

	#[cfg(feature = "database")]
	pub use gotham_middleware_diesel::Repo;

	#[cfg(feature = "openapi")]
	pub use indexmap::IndexMap;
	#[cfg(feature = "openapi")]
	pub use openapi_type::{openapi, OpenapiSchema, OpenapiType};
}

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "auth")]
pub use auth::{AuthHandler, AuthMiddleware, AuthSource, AuthStatus, AuthValidation, StaticAuthHandler};

#[cfg(feature = "cors")]
pub mod cors;
#[cfg(feature = "cors")]
pub use cors::{handle_cors, CorsConfig, CorsRoute};

#[cfg(feature = "openapi")]
mod openapi;
#[cfg(feature = "openapi")]
pub use openapi::{builder::OpenapiInfo, router::GetOpenapi};

mod endpoint;
#[cfg(feature = "openapi")]
pub use endpoint::EndpointWithSchema;
pub use endpoint::{Endpoint, NoopExtractor};

mod response;
pub use response::{
	AuthError, AuthError::Forbidden, AuthErrorOrOther, AuthResult, AuthSuccess, IntoResponse, IntoResponseError, NoContent,
	Raw, Redirect, Response, Success
};
#[cfg(feature = "openapi")]
pub use response::{IntoResponseWithSchema, ResponseSchema};

mod routing;
pub use routing::{DrawResourceRoutes, DrawResources};
#[cfg(feature = "openapi")]
pub use routing::{DrawResourceRoutesWithSchema, DrawResourcesWithSchema, WithOpenapi};

mod types;
pub use types::{FromBody, RequestBody, ResponseBody};

/// This trait must be implemented for every resource. It allows you to register the different
/// endpoints that can be handled by this resource to be registered with the underlying router.
///
/// It is not recommended to implement this yourself, just use `#[derive(Resource)]`.
#[_private_openapi_trait(ResourceWithSchema)]
pub trait Resource {
	/// Register all methods handled by this resource with the underlying router.
	#[openapi_bound("D: crate::DrawResourceRoutesWithSchema")]
	#[non_openapi_bound("D: crate::DrawResourceRoutes")]
	fn setup<D>(route: D);
}
