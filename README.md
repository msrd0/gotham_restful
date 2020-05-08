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
    <a href="https://blog.rust-lang.org/2020/03/12/Rust-1.42.html">
        <img alt="Minimum Rust Version" src="https://img.shields.io/badge/rustc-1.42+-orange.svg"/>
    </a>
	<a href="https://deps.rs/repo/gitlab/msrd0/gotham-restful">
		<img alt="dependencies" src="https://deps.rs/repo/gitlab/msrd0/gotham-restful/status.svg"/>
	</a>
</div>
<br/>

**Note:** The `stable` branch contains some bugfixes against the last release. The `master`
branch currently tracks gotham's master branch and the next release will use gotham 0.5.0 and be
compatible with the new future / async stuff.

This crate is an extension to the popular [gotham web framework][gotham] for Rust. It allows you to
create resources with assigned methods that aim to be a more convenient way of creating handlers
for requests.

## Design Goals

This is an opinionated framework on top of [gotham]. Unless your web server handles mostly JSON as
request/response bodies and does that in a RESTful way, this framework is probably a bad fit for
your application. The ultimate goal of gotham-restful is to provide a way to write a RESTful
web server in Rust as convenient as possible with the least amount of boilerplate neccessary.

## Methods

Assuming you assign `/foobar` to your resource, you can implement the following methods:

| Method Name | Required Arguments | HTTP Verb | HTTP Path      |
| ----------- | ------------------ | --------- | -----------    |
| read_all    |                    | GET       | /foobar        |
| read        | id                 | GET       | /foobar/:id    |
| search      | query              | GET       | /foobar/search |
| create      | body               | POST      | /foobar        |
| change_all  | body               | PUT       | /foobar        |
| change      | id, body           | PUT       | /foobar/:id    |
| remove_all  |                    | DELETE    | /foobar        |
| remove      | id                 | DELETE    | /foobar/:id    |

Each of those methods has a macro that creates the neccessary boilerplate for the Resource. A
simple example could look like this:

```rust
/// Our RESTful resource.
#[derive(Resource)]
#[resource(read)]
struct FooResource;

/// The return type of the foo read method.
#[derive(Serialize)]
struct Foo {
	id: u64
}

/// The foo read method handler.
#[read(FooResource)]
fn read(id: u64) -> Success<Foo> {
	Foo { id }.into()
}
```

## Arguments

Some methods require arguments. Those should be
 * **id** Should be a deserializable json-primitive like `i64` or `String`.
 * **body** Should be any deserializable object, or any type implementing [`RequestBody`].
 * **query** Should be any deserializable object whose variables are json-primitives. It will
   however not be parsed from json, but from HTTP GET parameters like in `search?id=1`. The
   type needs to implement [`QueryStringExtractor`].

Additionally, non-async handlers may take a reference to gotham's [`State`]. If you need to
have an async handler (that is, the function that the method macro is invoked on is declared
as `async fn`), consider returning the boxed future instead. Since [`State`] does not implement
`Sync` there is unfortunately no more convenient way.

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

#[create(ImageResource)]
fn create(body : RawImage) -> Raw<Vec<u8>> {
	Raw::new(body.content, body.content_type)
}
```

## Features

To make life easier for common use-cases, this create offers a few features that might be helpful
when you implement your web server.

### Authentication Feature

In order to enable authentication support, enable the `auth` feature gate. This allows you to
register a middleware that can automatically check for the existence of an JWT authentication
token. Besides being supported by the method macros, it supports to lookup the required JWT secret
with the JWT data, hence you can use several JWT secrets and decide on the fly which secret to use.
None of this is currently supported by gotham's own JWT middleware.

A simple example that uses only a single secret could look like this:

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

#[read(SecretResource)]
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

### Database Feature

The database feature allows an easy integration of [diesel] into your handler functions. Please
note however that due to the way gotham's diesel middleware implementation, it is not possible
to run async code while holding a database connection. If you need to combine async and database,
you'll need to borrow the connection from the [`State`] yourself and return a boxed future.

A simple non-async example could look like this:

```rust
#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[derive(Queryable, Serialize)]
struct Foo {
	id: i64,
	value: String
}

#[read_all(FooResource)]
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

## Examples

There is a lack of good examples, but there is currently a collection of code in the [example]
directory, that might help you. Any help writing more examples is highly appreciated.

## License

Licensed under your option of:
 - [Apache License Version 2.0](https://gitlab.com/msrd0/gotham-restful/blob/master/LICENSE-Apache)
 - [Eclipse Public License Version 2.0](https://gitlab.com/msrd0/gotham-restful/blob/master/LICENSE-EPL)


 [diesel]: https://diesel.rs/
 [example]: https://gitlab.com/msrd0/gotham-restful/tree/master/example
 [gotham]: https://gotham.rs/
 [serde_json]: https://github.com/serde-rs/json#serde-json----
 [`QueryStringExtractor`]: ../gotham/extractor/trait.QueryStringExtractor.html
 [`RequestBody`]: trait.RequestBody.html
 [`State`]: ../gotham/state/struct.State.html
