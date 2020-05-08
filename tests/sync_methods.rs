#[macro_use] extern crate gotham_derive;

use gotham::{
	router::builder::*,
	test::TestServer
};
use gotham_restful::*;
use mime::{APPLICATION_JSON, TEXT_PLAIN};
use serde::Deserialize;

mod util { include!("util/mod.rs"); }
use util::{test_get_response, test_post_response, test_put_response, test_delete_response};


#[derive(Resource)]
#[resource(read_all, read, search, create, change_all, change, remove_all, remove)]
struct FooResource;

#[derive(Deserialize)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
#[allow(dead_code)]
struct FooBody
{
	data : String
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
#[allow(dead_code)]
struct FooSearch
{
	query : String
}

const READ_ALL_RESPONSE : &[u8] = b"1ARwwSPVyOKpJKrYwqGgECPVWDl1BqajAAj7g7WJ3e";
#[read_all(FooResource)]
fn read_all() -> Raw<&'static [u8]>
{
	Raw::new(READ_ALL_RESPONSE, TEXT_PLAIN)
}

const READ_RESPONSE : &[u8] = b"FEReHoeBKU17X2bBpVAd1iUvktFL43CDu0cFYHdaP9";
#[read(FooResource)]
fn read(_id : u64) -> Raw<&'static [u8]>
{
	Raw::new(READ_RESPONSE, TEXT_PLAIN)
}

const SEARCH_RESPONSE : &[u8] = b"AWqcQUdBRHXKh3at4u79mdupOAfEbnTcx71ogCVF0E";
#[search(FooResource)]
fn search(_body : FooSearch) -> Raw<&'static [u8]>
{
	Raw::new(SEARCH_RESPONSE, TEXT_PLAIN)
}

const CREATE_RESPONSE : &[u8] = b"y6POY7wOMAB0jBRBw0FJT7DOpUNbhmT8KdpQPLkI83";
#[create(FooResource)]
fn create(_body : FooBody) -> Raw<&'static [u8]>
{
	Raw::new(CREATE_RESPONSE, TEXT_PLAIN)
}

const CHANGE_ALL_RESPONSE : &[u8] = b"QlbYg8gHE9OQvvk3yKjXJLTSXlIrg9mcqhfMXJmQkv";
#[change_all(FooResource)]
fn change_all(_body : FooBody) -> Raw<&'static [u8]>
{
	Raw::new(CHANGE_ALL_RESPONSE, TEXT_PLAIN)
}

const CHANGE_RESPONSE : &[u8] = b"qGod55RUXkT1lgPO8h0uVM6l368O2S0GrwENZFFuRu";
#[change(FooResource)]
fn change(_id : u64, _body : FooBody) -> Raw<&'static [u8]>
{
	Raw::new(CHANGE_RESPONSE, TEXT_PLAIN)
}

const REMOVE_ALL_RESPONSE : &[u8] = b"Y36kZ749MRk2Nem4BedJABOZiZWPLOtiwLfJlGTwm5";
#[remove_all(FooResource)]
fn remove_all() -> Raw<&'static [u8]>
{
	Raw::new(REMOVE_ALL_RESPONSE, TEXT_PLAIN)
}

const REMOVE_RESPONSE : &[u8] = b"CwRzBrKErsVZ1N7yeNfjZuUn1MacvgBqk4uPOFfDDq";
#[remove(FooResource)]
fn remove(_id : u64) -> Raw<&'static [u8]>
{
	Raw::new(REMOVE_RESPONSE, TEXT_PLAIN)
}

#[test]
fn sync_methods()
{
	let server = TestServer::new(build_simple_router(|router| {
		router.resource::<FooResource>("foo");
	})).unwrap();
	
	test_get_response(&server, "http://localhost/foo", READ_ALL_RESPONSE);
	test_get_response(&server, "http://localhost/foo/1", READ_RESPONSE);
	test_get_response(&server, "http://localhost/foo/search?query=hello+world", SEARCH_RESPONSE);
	test_post_response(&server, "http://localhost/foo", r#"{"data":"hello world"}"#, APPLICATION_JSON, CREATE_RESPONSE);
	test_put_response(&server, "http://localhost/foo", r#"{"data":"hello world"}"#, APPLICATION_JSON, CHANGE_ALL_RESPONSE);
	test_put_response(&server, "http://localhost/foo/1", r#"{"data":"hello world"}"#, APPLICATION_JSON, CHANGE_RESPONSE);
	test_delete_response(&server, "http://localhost/foo", REMOVE_ALL_RESPONSE);
	test_delete_response(&server, "http://localhost/foo/1", REMOVE_RESPONSE);
}
