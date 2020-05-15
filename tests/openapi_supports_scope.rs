#![cfg(feature = "openapi")]
use gotham::{
	router::builder::*,
	test::TestServer
};
use gotham_restful::*;
use mime::TEXT_PLAIN;

#[allow(dead_code)]
mod util { include!("util/mod.rs"); }
use util::test_get_response;


const RESPONSE : &[u8] = b"This is the only valid response.";

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all(FooResource)]
fn read_all() -> Raw<&'static [u8]>
{
	Raw::new(RESPONSE, TEXT_PLAIN)
}


#[test]
fn openapi_supports_scope()
{
	let info = OpenapiInfo {
		title: "Test".to_owned(),
		version: "1.2.3".to_owned(),
		urls: Vec::new()
	};
	let server = TestServer::new(build_simple_router(|router| {
		router.with_openapi(info, |mut router| {
			router.resource::<FooResource>("foo1");
			router.scope("/bar", |router| {
				router.resource::<FooResource>("foo2");
				router.scope("/baz", |router| {
					router.resource::<FooResource>("foo3");
				})
			});
			router.resource::<FooResource>("foo4");
		});
	})).unwrap();
	
	test_get_response(&server, "http://localhost/foo1", RESPONSE);
	test_get_response(&server, "http://localhost/bar/foo2", RESPONSE);
	test_get_response(&server, "http://localhost/bar/baz/foo3", RESPONSE);
	test_get_response(&server, "http://localhost/foo4", RESPONSE);
}
