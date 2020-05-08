use gotham::{
	hyper::Body,
	test::TestServer
};
use mime::Mime;

pub fn test_get_response(server : &TestServer, path : &str, expected : &[u8])
{
	let res = server.client().get(path).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

pub fn test_post_response<B>(server : &TestServer, path : &str, body : B, mime : Mime, expected : &[u8])
where
	B : Into<Body>
{
	let res = server.client().post(path, body, mime).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

pub fn test_put_response<B>(server : &TestServer, path : &str, body : B, mime : Mime, expected : &[u8])
where
	B : Into<Body>
{
	let res = server.client().put(path, body, mime).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

pub fn test_delete_response(server : &TestServer, path : &str, expected : &[u8])
{
	let res = server.client().delete(path).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}
