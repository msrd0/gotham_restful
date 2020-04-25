<div align="center">
	<h1>gotham-restful</h1>
</div>
<div align="center">
	<a href="https://gitlab.com/msrd0/gotham-restful/-/commits/master">
		<img alt="pipeline status" src="https://gitlab.com/msrd0/gotham-restful/badges/master/pipeline.svg"/>
	</a>
	<a href="https://gitlab.com/msrd0/gotham-restful/-/commits/master">
		<img alt="coverage report" src="https://gitlab.com/msrd0/gotham-restful/badges/master/coverage.svg"/>
	</a>
	<a href="https://crates.io/crates/gotham_restful">
        <img alt="crates.io" src="https://img.shields.io/crates/v/gotham_restful.svg"/>
    </a>
	<a href="https://docs.rs/crate/gotham_restful">
        <img alt="docs.rs" src="https://docs.rs/gotham_restful/badge.svg"/>
    </a>
	<a href="https://www.rust-lang.org/en-US/">
        <img alt="Build with Rust" src="https://img.shields.io/badge/Made%20with-Rust-orange.svg"/>
    </a>
    <a href="https://blog.rust-lang.org/2020/03/12/Rust-1.42.html">
        <img alt="Minimum Rust Version" src="https://img.shields.io/badge/rustc-1.42+-yellow.svg"/>
    </a>
</div>
<br/>

This crate is an extension to the popular [gotham web framework][gotham] for Rust. The idea is to
have several RESTful resources that can be added to the gotham router. This crate will take care
of everything else, like parsing path/query parameters, request bodies, and writing response
bodies, relying on [`serde`][serde] and [`serde_json`][serde_json] for (de)serializing. If you
enable the `openapi` feature, you can also generate an OpenAPI Specification from your RESTful
resources.

**Note:** The `stable` branch contains some bugfixes against the last release. The `master`
branch currently tracks gotham's master branch and the next release will use gotham 0.5.0 and be
compatible with the new future / async stuff.

## Usage

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
struct RawImage {
	content: Vec<u8>,
	content_type: Mime
}

#[rest_create(ImageResource)]
fn create(_state : &mut State, body : RawImage) -> Raw<Vec<u8>> {
	Raw::new(body.content, body.content_type)
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
