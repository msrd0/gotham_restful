#![cfg(all(feature = "auth", feature = "chrono", feature = "openapi"))]

#[macro_use] extern crate gotham_derive;

use chrono::{NaiveDate, NaiveDateTime};
use gotham::{
	pipeline::{new_pipeline, single::single_pipeline},
	router::builder::*,
	test::TestServer
};
use gotham_restful::*;
use mime::IMAGE_PNG;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
mod util { include!("util/mod.rs"); }
use util::{test_get_response, test_openapi_response};


const IMAGE_RESPONSE : &[u8] = b"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABAQMAAAAl21bKAAAAA1BMVEUA/wA0XsCoAAAAAXRSTlN/gFy0ywAAAApJREFUeJxjYgAAAAYAAzY3fKgAAAAASUVORK5CYII=";

#[derive(Resource)]
#[resource(read, change)]
struct ImageResource;

#[derive(FromBody, RequestBody)]
#[supported_types(IMAGE_PNG)]
struct Image(Vec<u8>);

#[read(ImageResource, operation_id = "getImage")]
fn get_image(_id : u64) -> Raw<&'static [u8]>
{
	Raw::new(IMAGE_RESPONSE, "image/png;base64".parse().unwrap())
}

#[change(ImageResource, operation_id = "setImage")]
fn set_image(_id : u64, _image : Image)
{
}


#[derive(Resource)]
#[resource(read, search)]
struct SecretResource;

#[derive(Deserialize, Clone)]
struct AuthData
{
	sub : String,
	iat : u64,
	exp : u64
}

type AuthStatus = gotham_restful::AuthStatus<AuthData>;

#[derive(OpenapiType, Serialize)]
struct Secret
{
	code : f32
}

#[derive(OpenapiType, Serialize)]
struct Secrets
{
	secrets : Vec<Secret>
}

#[derive(Deserialize, OpenapiType, StateData, StaticResponseExtender)]
struct SecretQuery
{
	date : NaiveDate,
	hour : Option<u16>,
	minute : Option<u16>
}

#[read(SecretResource)]
fn read_secret(auth : AuthStatus, _id : NaiveDateTime) -> AuthSuccess<Secret>
{
	auth.ok()?;
	Ok(Secret { code: 4.2 })
}

#[search(SecretResource)]
fn search_secret(auth : AuthStatus, _query : SecretQuery) -> AuthSuccess<Secrets>
{
	auth.ok()?;
	Ok(Secrets {
		secrets: vec![Secret { code: 4.2 }, Secret { code: 3.14 }]
	})
}


#[test]
fn openapi_supports_scope()
{
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
			router.get_openapi("openapi");
			router.resource::<SecretResource>("secret");
		});
	})).unwrap();
	
	test_openapi_response(&server, "http://localhost/openapi", "tests/openapi_specification.json");
}
