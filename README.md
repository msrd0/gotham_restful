# gotham_restful

[![Build Status](https://gitlab.com/msrd0/gotham-restful/badges/master/pipeline.svg)](https://gitlab.com/msrd0/gotham-restful/commits/master)
[![Coverage Status](https://codecov.io/gl/msrd0/gotham-restful/branch/master/graph/badge.svg)](https://codecov.io/gl/msrd0/gotham-restful)

This crate is an extension to the popular [gotham web framework][gotham] for Rust. The idea is to
have several RESTful resources that can be added to the gotham router. This crate will take care
of everything else, like parsing path/query parameters, request bodies, and writing response
bodies, relying on [`serde`][serde] and [`serde_json`][serde_json] for (de)serializing. If you
enable the `openapi` feature, you can also generate an OpenAPI Specification from your RESTful
resources.

## Usage

This crate targets stable rust, currently requiring rustc 1.40+. To use this crate, add the
following to your `Cargo.toml`:

```toml
[dependencies]
gotham_restful = "0.0.1"
```

A basic server with only one resource, handling a simple `GET` request, could look like this:

```rust
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
		route.resource::<UsersResource>("users");
	}));
}
```

Uploads and Downloads can also be handled, but you need to specify the mime type manually:

```rust
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
```

Look at the [example] for more methods and usage with the `openapi` feature.

## Known Issues

These are currently known major issues. For a complete list please see
[the issue tracker](https://gitlab.com/msrd0/gotham-restful/issues).
If you encounter any issues that aren't yet reported, please report them
[here](https://gitlab.com/msrd0/gotham-restful/issues/new).

 - Enabling the `openapi` feature might break code ([#4](https://gitlab.com/msrd0/gotham-restful/issues/4))
 - For `chrono`'s `DateTime` types, the format is `date-time` instead of `datetime` ([openapiv3#14](https://github.com/glademiller/openapiv3/pull/14))

## License

Licensed under your option of:
 - [Apache License Version 2.0](https://gitlab.com/msrd0/gotham-restful/blob/master/LICENSE-Apache)
 - [Eclipse Public License Version 2.0](https://gitlab.com/msrd0/gotham-restful/blob/master/LICENSE-EPL)


[example]: https://gitlab.com/msrd0/gotham-restful/tree/master/example
[gotham]: https://gotham.rs/
[serde]: https://github.com/serde-rs/serde#serde-----
[serde_json]: https://github.com/serde-rs/json#serde-json----
