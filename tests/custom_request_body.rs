use gotham::{
	hyper::header::CONTENT_TYPE,
	router::builder::*,
	test::TestServer
};
use gotham_restful::*;
use mime::TEXT_PLAIN;


const RESPONSE : &[u8] = b"This is the only valid response.";

#[derive(Resource)]
#[resource(create)]
struct FooResource;

#[derive(FromBody, RequestBody)]
#[supported_types(TEXT_PLAIN)]
struct Foo {
	content: Vec<u8>,
	content_type: Mime
}

#[create(FooResource)]
fn create(body : Foo) -> Raw<Vec<u8>> {
	Raw::new(body.content, body.content_type)
}


#[test]
fn custom_request_body()
{
	let server = TestServer::new(build_simple_router(|router| {
		router.resource::<FooResource>("foo");
	})).unwrap();
	
	let res = server.client()
		.post("http://localhost/foo", RESPONSE, TEXT_PLAIN)
		.perform().unwrap();
	assert_eq!(res.headers().get(CONTENT_TYPE).unwrap().to_str().unwrap(), "text/plain");
	let res = res.read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, RESPONSE);
}
