use gotham::{
	hyper::Body,
	test::TestServer
};
use mime::Mime;
#[allow(unused_imports)]
use std::{fs::File, io::{Read, Write}, str};

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

#[cfg(feature = "openapi")]
pub fn test_openapi_response(server : &TestServer, path : &str, output_file : &str)
{
	let res = server.client().get(path).perform().unwrap().read_body().unwrap();
	let body = serde_json::to_string_pretty(&serde_json::from_slice::<serde_json::Value>(res.as_ref()).unwrap()).unwrap();
	match File::open(output_file) {
		Ok(mut file) => {
			let mut expected = String::new();
			file.read_to_string(&mut expected).unwrap();
			assert_eq!(body, expected);
		},
		Err(_) => {
			let mut file = File::create(output_file).unwrap();
			file.write_all(body.as_bytes()).unwrap();
		}
	};
}
