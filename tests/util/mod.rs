use gotham::{
	hyper::Body,
	test::TestServer
};
use log::info;
use gotham::mime::Mime;
#[allow(unused_imports)]
use std::{fs::File, io::{Read, Write}, str};

pub fn test_get_response(server : &TestServer, path : &str, expected : &[u8])
{
	info!("GET {path}");
	let res = server.client().get(path).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

pub fn test_post_response<B>(server : &TestServer, path : &str, body : B, mime : Mime, expected : &[u8])
where
	B : Into<Body>
{
	info!("POST {path}");
	let res = server.client().post(path, body, mime).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

pub fn test_put_response<B>(server : &TestServer, path : &str, body : B, mime : Mime, expected : &[u8])
where
	B : Into<Body>
{
	info!("PUT {path}");
	let res = server.client().put(path, body, mime).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

pub fn test_delete_response(server : &TestServer, path : &str, expected : &[u8])
{
	info!("DELETE {path}");
	let res = server.client().delete(path).perform().unwrap().read_body().unwrap();
	let body : &[u8] = res.as_ref();
	assert_eq!(body, expected);
}

#[cfg(feature = "openapi")]
pub fn test_openapi_response(server : &TestServer, path : &str, output_file : &str)
{
	info!("GET {path}");
	let res = server.client().get(path).perform().unwrap().read_body().unwrap();
	let body: serde_json::Value = serde_json::from_slice(&res).unwrap();

	let mut file = File::open(output_file).unwrap();
	let expected: serde_json::Value = serde_json::from_reader(&mut file).unwrap();

	//eprintln!("{body}");
	assert_eq!(body, expected);
}
