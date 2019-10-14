# gotham_restful

[![Build Status](https://gitlab.com/msrd0/gotham-restful/badges/master/build.svg)](https://gitlab.com/msrd0/gotham-restful/commits/master)
[![Coverage Status](https://codecov.io/gl/msrd0/gotham-restful/branch/master/graph/badge.svg)](https://codecov.io/gl/msrd0/gotham-restful)

This crate is an extension to the popular [gotham web framework][gotham] for Rust. The idea is to
have several RESTful resources that can be added to the gotham router. This crate will take care
of everything else, like parsing path/query parameters, request bodies, and writing response
bodies, relying on [`serde`][serde] and [`serde_json`][serde_json] for (de)serializing. If you
enable the `openapi` feature, you can also generate an OpenAPI Specification from your RESTful
resources.

## Usage

To use this crate, add the following to your `Cargo.toml`:

```toml
[dependencies]
gotham_restful = "0.0.1"
```

A basic server with only one resource, handling a simple `GET` request, could look like this:

```rust
#
/// Our RESTful Resource.
#[derive(Resource)]
#[rest_resource(read_all)]
struct UsersResource;

/// Our return type.
#[derive(Deserialize, Serialize)]
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

Look at the [example] for more methods and usage with the `openapi` feature.

## License

THE ACCOMPANYING PROGRAM IS PROVIDED UNDER THE TERMS OF THE ECLIPSE <br/>
PUBLIC LICENSE VERSION 2.0. ANY USE, REPRODUCTION OR DISTRIBUTION <br/>
OF THE PROGRAM CONSTITUTES RECIPIENT'S ACCEPTANCE OF THIS LICENSE.


[example]: https://gitlab.com/msrd0/gotham-restful/tree/master/example
[gotham]: https://gotham.rs/
[serde]: https://github.com/serde-rs/serde#serde-----
[serde_json]: https://github.com/serde-rs/json#serde-json----
