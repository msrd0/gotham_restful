#![cfg(all(feature = "auth", feature = "openapi"))]

#[macro_use]
extern crate gotham_derive;
#[macro_use]
extern crate pretty_assertions;

use gotham::{
	hyper::Method,
	pipeline::{new_pipeline, single::single_pipeline},
	router::builder::*,
	test::TestServer
};
use gotham_restful::*;
use mime::IMAGE_PNG;
use openapi_type::OpenapiType;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
mod util {
	include!("util/mod.rs");
}
use util::{test_get_response, test_openapi_response};

const IMAGE_RESPONSE : &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEUA/wA0XsCoAAAAAXRSTlN/gFy0ywAAAApJREFUeJxjYgAAAAYAAzY3fKgAAAAASUVORK5CYII=";

#[derive(Resource)]
#[resource(get_image, set_image)]
struct ImageResource;

#[derive(FromBody, RequestBody)]
#[supported_types(IMAGE_PNG)]
struct Image(Vec<u8>);

#[read(operation_id = "getImage")]
fn get_image(_id: u64) -> Raw<&'static [u8]> {
	Raw::new(IMAGE_RESPONSE, "image/png;base64".parse().unwrap())
}

#[change(operation_id = "setImage")]
fn set_image(_id: u64, _image: Image) {}

#[derive(Resource)]
#[resource(read_secret, search_secret)]
struct SecretResource;

#[derive(Deserialize, Clone)]
struct AuthData {
	sub: String,
	iat: u64,
	exp: u64
}

type AuthStatus = gotham_restful::AuthStatus<AuthData>;

#[derive(OpenapiType, Serialize)]
struct Secret {
	code: f32
}

#[derive(OpenapiType, Serialize)]
struct Secrets {
	secrets: Vec<Secret>
}

#[derive(Clone, Deserialize, OpenapiType, StateData, StaticResponseExtender)]
struct SecretQuery {
	date: String,
	hour: Option<u16>,
	minute: Option<u16>
}

/// This endpoint gives access to the secret.
///
/// You need to be authenticated to call this endpoint.
#[read]
fn read_secret(auth: AuthStatus, _id: String) -> AuthSuccess<Secret> {
	auth.ok()?;
	Ok(Secret { code: 4.2 })
}

#[search]
fn search_secret(auth: AuthStatus, _query: SecretQuery) -> AuthSuccess<Secrets> {
	auth.ok()?;
	Ok(Secrets {
		secrets: vec![Secret { code: 4.2 }, Secret { code: 3.14 }]
	})
}

#[derive(Resource)]
#[resource(custom_read_with, custom_patch)]
struct CustomResource;

#[derive(Clone, Deserialize, OpenapiType, StateData, StaticResponseExtender)]
struct ReadWithPath {
	from: String,
	id: u64
}

#[endpoint(method = "Method::GET", uri = "read/:from/with/:id")]
fn custom_read_with(_path: ReadWithPath) {}

#[endpoint(method = "Method::PATCH", uri = "", body = true)]
fn custom_patch(_body: String) {}

#[test]
fn openapi_specification() {
	let info = OpenapiInfo {
		title: "This is just a test".to_owned(),
		version: "1.2.3".to_owned(),
		urls: vec!["http://localhost:12345/api/v1".to_owned()]
	};
	let auth: AuthMiddleware<AuthData, _> = AuthMiddleware::new(
		AuthSource::AuthorizationHeader,
		AuthValidation::default(),
		StaticAuthHandler::from_array(b"zlBsA2QXnkmpe0QTh8uCvtAEa4j33YAc")
	);
	let (chain, pipelines) = single_pipeline(new_pipeline().add(auth).build());
	let server = TestServer::new(build_router(chain, pipelines, |router| {
		router.with_openapi(info, |mut router| {
			router.resource::<ImageResource>("img");
			router.resource::<SecretResource>("secret");
			router.resource::<CustomResource>("custom");
			router.get_openapi("openapi");
		});
	}))
	.unwrap();

	test_openapi_response(&server, "http://localhost/openapi", "tests/openapi_specification.json");
}
