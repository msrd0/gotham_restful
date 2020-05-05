#[cfg(feature = "openapi")]
mod openapi_supports_scope
{


use gotham::{
	router::builder::*,
	test::TestServer
};
use gotham_restful::*;
use mime::TEXT_PLAIN;

const RESPONSE : &[u8] = b"This is the only valid response.";

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all(FooResource)]
fn read_all() -> Raw<&'static [u8]>
{
	Raw::new(RESPONSE, TEXT_PLAIN)
}


fn test_response(server : &TestServer, path : &str)
{
	let res = server.client().get(path).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, RESPONSE);
}

#[test]
fn test()
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
	
	test_response(&server, "http://localhost/foo1");
	test_response(&server, "http://localhost/bar/foo2");
	test_response(&server, "http://localhost/bar/baz/foo3");
	test_response(&server, "http://localhost/foo4");
}


} // mod test
