<div align="center">
	<h1>gotham-restful</h1>
</div>
<div align="center">
	<a href="https://gitlab.com/msrd0/gotham-restful/pipelines">
		<img alt="pipeline status" src="https://gitlab.com/msrd0/gotham-restful/badges/master/pipeline.svg"/>
	</a>
	<a href="https://msrd0.gitlab.io/gotham-restful/coverage.html">
		<img alt="coverage report" src="https://gitlab.com/msrd0/gotham-restful/badges/master/coverage.svg"/>
	</a>
	<a href="https://crates.io/crates/gotham_restful">
        <img alt="crates.io" src="https://img.shields.io/crates/v/gotham_restful.svg"/>
    </a>
	<a href="https://docs.rs/crate/gotham_restful">
        <img alt="docs.rs" src="https://docs.rs/gotham_restful/badge.svg"/>
    </a>
	<a href="https://msrd0.gitlab.io/gotham-restful/gotham_restful/index.html">
		<img alt="rustdoc" src="https://img.shields.io/badge/docs-master-blue.svg"/>
	</a>
    <a href="https://blog.rust-lang.org/2020/04/23/Rust-1.43.0.html">
        <img alt="Minimum Rust Version" src="https://img.shields.io/badge/rustc-1.43+-orange.svg"/>
    </a>
	<a href="https://deps.rs/repo/gitlab/msrd0/gotham-restful">
		<img alt="dependencies" src="https://deps.rs/repo/gitlab/msrd0/gotham-restful/status.svg"/>
	</a>
</div>
<br/>

This crate is an extension to the popular [gotham web framework][gotham] for Rust. It allows you to
create resources with assigned endpoints that aim to be a more convenient way of creating handlers
for requests.

## Features

 - Automatically parse **JSON** request and produce response bodies
 - Allow using **raw** request and response bodies
 - Convenient **macros** to create responses that can be registered with gotham's router
 - Auto-Generate an **OpenAPI** specification for your API
 - Manage **CORS** headers so you don't have to
 - Manage **Authentication** with JWT
 - Integrate diesel connection pools for easy **database** integration

## Safety

This crate is just as safe as you'd expect from anything written in safe Rust - and
`#![forbid(unsafe_code)]` ensures that no unsafe was used.

## Endpoints

There are a set of pre-defined endpoints that should cover the majority of REST APIs. However,
it is also possible to define your own endpoints.

### Pre-defined Endpoints

Assuming you assign `/foobar` to your resource, the following pre-defined endpoints exist:

| Endpoint Name | Required Arguments | HTTP Verb | HTTP Path      |
| ------------- | ------------------ | --------- | -------------- |
| read_all      |                    | GET       | /foobar        |
| read          | id                 | GET       | /foobar/:id    |
| search        | query              | GET       | /foobar/search |
| create        | body               | POST      | /foobar        |
| change_all    | body               | PUT       | /foobar        |
| change        | id, body           | PUT       | /foobar/:id    |
| remove_all    |                    | DELETE    | /foobar        |
| remove        | id                 | DELETE    | /foobar/:id    |

Each of those endpoints has a macro that creates the neccessary boilerplate for the Resource. A
simple example looks like this:

```rust
/// Our RESTful resource.
#[derive(Resource)]
#[resource(read)]
struct FooResource;

/// The return type of the foo read endpoint.
#[derive(Serialize)]
struct Foo {
	id: u64
}

/// The foo read endpoint.
#[read]
fn read(id: u64) -> Success<Foo> {
	Foo { id }.into()
}
```

### Custom Endpoints

Defining custom endpoints is done with the `#[endpoint]` macro. The syntax is similar to that
of the pre-defined endpoints, but you need to give it more context:

```rust
use gotham_restful::gotham::hyper::Method;

#[derive(Resource)]
#[resource(custom_endpoint)]
struct CustomResource;

/// This type is used to parse path parameters.
#[derive(Deserialize, StateData, StaticResponseExtender)]
struct CustomPath {
	name: String
}

#[endpoint(uri = "custom/:name/read", method = "Method::GET", params = false, body = false)]
fn custom_endpoint(path: CustomPath) -> Success<String> {
	path.name.into()
}
```

## Arguments

Some endpoints require arguments. Those should be
 * **id** Should be a deserializable json-primitive like [`i64`] or [`String`].
 * **body** Should be any deserializable object, or any type implementing [`RequestBody`].
 * **query** Should be any deserializable object whose variables are json-primitives. It will
   however not be parsed from json, but from HTTP GET parameters like in `search?id=1`. The
   type needs to implement [`QueryStringExtractor`](gotham::extractor::QueryStringExtractor).

Additionally, all handlers may take a reference to gotham's [`State`]. Please note that for async
handlers, it needs to be a mutable reference until rustc's lifetime checks across await bounds
improve.

## Uploads and Downloads

By default, every request body is parsed from json, and every respone is converted to json using
[serde_json]. However, you may also use raw bodies. This is an example where the request body
is simply returned as the response again, no json parsing involved:

```rust
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
```

## Features

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

### Authentication Feature

In order to enable authentication support, enable the `auth` feature gate. This allows you to
register a middleware that can automatically check for the existence of an JWT authentication
token. Besides being supported by the endpoint macros, it supports to lookup the required JWT secret
with the JWT data, hence you can use several JWT secrets and decide on the fly which secret to use.
None of this is currently supported by gotham's own JWT middleware.

A simple example that uses only a single secret looks like this:

```rust
#[derive(Resource)]
#[resource(read)]
struct SecretResource;

#[derive(Serialize)]
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
```

### CORS Feature

The cors feature allows an easy usage of this web server from other origins. By default, only
the `Access-Control-Allow-Methods` header is touched. To change the behaviour, add your desired
configuration as a middleware.

A simple example that allows authentication from every origin (note that `*` always disallows
authentication), and every content type, looks like this:

```rust
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
```

The cors feature can also be used for non-resource handlers. Take a look at [`CorsRoute`]
for an example.

### Database Feature

The database feature allows an easy integration of [diesel] into your handler functions. Please
note however that due to the way gotham's diesel middleware implementation, it is not possible
to run async code while holding a database connection. If you need to combine async and database,
you'll need to borrow the connection from the [`State`] yourself and return a boxed future.

A simple non-async example looks like this:

```rust
#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[derive(Queryable, Serialize)]
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
```

### OpenAPI Feature

The OpenAPI feature is probably the most powerful one of this crate. Definitely read this section
carefully both as a binary as well as a library author to avoid unwanted suprises.

In order to automatically create an openapi specification, gotham-restful needs knowledge over
all routes and the types returned. `serde` does a great job at serialization but doesn't give
enough type information, so all types used in the router need to implement `OpenapiType`. This
can be derived for almoust any type and there should be no need to implement it manually. A simple
example looks like this:

```rust
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
			route.get_openapi("openapi");
		});
	}));
}
```

Above example adds the resource as before, but adds another endpoint that we specified as `/openapi`.
It will return the generated openapi specification in JSON format. This allows you to easily write
clients in different languages without worying to exactly replicate your api in each of those
languages.

However, please note that by default, the `without-openapi` feature of this crate is enabled.
Disabling it in favour of the `openapi` feature will add an additional type bound, [`OpenapiType`],
on some of the types in [`Endpoint`] and related traits. This means that some code might only
compile on either feature, but not on both. If you are writing a library that uses gotham-restful,
it is strongly recommended to pass both features through and conditionally enable the openapi
code, like this:

```rust
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
struct Foo;
```

## Examples

This readme and the crate documentation contain some of example. In addition to that, there is
a collection of code in the [example] directory that might help you. Any help writing more
examples is highly appreciated.


 [diesel]: https://diesel.rs/
 [example]: https://gitlab.com/msrd0/gotham-restful/tree/master/example
 [gotham]: https://gotham.rs/
 [serde_json]: https://github.com/serde-rs/json#serde-json----
 [`State`]: gotham::state::State

## Versioning

Like all rust crates, this crate will follow semantic versioning guidelines. However, changing
the MSRV (minimum supported rust version) is not considered a breaking change.

## License

Copyright (C) 2020-2021 Dominic Meiser and [contributors](https://gitlab.com/msrd0/gotham-restful/-/graphs/master).

```
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

	https://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
```
