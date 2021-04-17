#[macro_use]
extern crate gotham_derive;

use gotham::{
	hyper::{HeaderMap, Method},
	router::builder::*,
	state::State,
	test::TestServer
};
use gotham_restful::*;
use mime::{APPLICATION_JSON, TEXT_PLAIN};
#[cfg(feature = "openapi")]
use openapi_type::OpenapiType;
use serde::Deserialize;
use tokio::time::{sleep, Duration};

mod util {
	include!("util/mod.rs");
}
use util::{test_delete_response, test_get_response, test_post_response, test_put_response};

#[derive(Resource)]
#[resource(read_all, read, search, create, update_all, update, delete_all, delete, state_test)]
struct FooResource;

#[derive(Deserialize)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
#[allow(dead_code)]
struct FooBody {
	data: String
}

#[derive(Clone, Deserialize, StateData, StaticResponseExtender)]
#[cfg_attr(feature = "openapi", derive(OpenapiType))]
#[allow(dead_code)]
struct FooSearch {
	query: String
}

const READ_ALL_RESPONSE: &[u8] = b"1ARwwSPVyOKpJKrYwqGgECPVWDl1BqajAAj7g7WJ3e";
#[read_all]
async fn read_all() -> Raw<&'static [u8]> {
	Raw::new(READ_ALL_RESPONSE, TEXT_PLAIN)
}

const READ_RESPONSE: &[u8] = b"FEReHoeBKU17X2bBpVAd1iUvktFL43CDu0cFYHdaP9";
#[read]
async fn read(_id: u64) -> Raw<&'static [u8]> {
	Raw::new(READ_RESPONSE, TEXT_PLAIN)
}

const SEARCH_RESPONSE: &[u8] = b"AWqcQUdBRHXKh3at4u79mdupOAfEbnTcx71ogCVF0E";
#[search]
async fn search(_body: FooSearch) -> Raw<&'static [u8]> {
	Raw::new(SEARCH_RESPONSE, TEXT_PLAIN)
}

const CREATE_RESPONSE: &[u8] = b"y6POY7wOMAB0jBRBw0FJT7DOpUNbhmT8KdpQPLkI83";
#[create]
async fn create(_body: FooBody) -> Raw<&'static [u8]> {
	Raw::new(CREATE_RESPONSE, TEXT_PLAIN)
}

const UPDATE_ALL_RESPONSE: &[u8] = b"QlbYg8gHE9OQvvk3yKjXJLTSXlIrg9mcqhfMXJmQkv";
#[update_all]
async fn update_all(_body: FooBody) -> Raw<&'static [u8]> {
	Raw::new(UPDATE_ALL_RESPONSE, TEXT_PLAIN)
}

const UPDATE_RESPONSE: &[u8] = b"qGod55RUXkT1lgPO8h0uVM6l368O2S0GrwENZFFuRu";
#[update]
async fn update(_id: u64, _body: FooBody) -> Raw<&'static [u8]> {
	Raw::new(UPDATE_RESPONSE, TEXT_PLAIN)
}

const DELETE_ALL_RESPONSE: &[u8] = b"Y36kZ749MRk2Nem4BedJABOZiZWPLOtiwLfJlGTwm5";
#[delete_all]
async fn delete_all() -> Raw<&'static [u8]> {
	Raw::new(DELETE_ALL_RESPONSE, TEXT_PLAIN)
}

const DELETE_RESPONSE: &[u8] = b"CwRzBrKErsVZ1N7yeNfjZuUn1MacvgBqk4uPOFfDDq";
#[delete]
async fn delete(_id: u64) -> Raw<&'static [u8]> {
	Raw::new(DELETE_RESPONSE, TEXT_PLAIN)
}

const STATE_TEST_RESPONSE: &[u8] = b"xxJbxOuwioqR5DfzPuVqvaqRSfpdNQGluIvHU4n1LM";
#[endpoint(method = "Method::GET", uri = "state_test")]
async fn state_test(state: &mut State) -> Raw<&'static [u8]> {
	sleep(Duration::from_nanos(1)).await;
	state.borrow::<HeaderMap>();
	sleep(Duration::from_nanos(1)).await;
	Raw::new(STATE_TEST_RESPONSE, TEXT_PLAIN)
}

#[test]
fn async_methods() {
	let _ = pretty_env_logger::try_init_timed();

	let server = TestServer::new(build_simple_router(|router| {
		router.resource::<FooResource>("foo");
	}))
	.unwrap();

	test_get_response(&server, "http://localhost/foo", READ_ALL_RESPONSE);
	test_get_response(&server, "http://localhost/foo/1", READ_RESPONSE);
	test_get_response(&server, "http://localhost/foo/search?query=hello+world", SEARCH_RESPONSE);
	test_post_response(
		&server,
		"http://localhost/foo",
		r#"{"data":"hello world"}"#,
		APPLICATION_JSON,
		CREATE_RESPONSE
	);
	test_put_response(
		&server,
		"http://localhost/foo",
		r#"{"data":"hello world"}"#,
		APPLICATION_JSON,
		UPDATE_ALL_RESPONSE
	);
	test_put_response(
		&server,
		"http://localhost/foo/1",
		r#"{"data":"hello world"}"#,
		APPLICATION_JSON,
		UPDATE_RESPONSE
	);
	test_delete_response(&server, "http://localhost/foo", DELETE_ALL_RESPONSE);
	test_delete_response(&server, "http://localhost/foo/1", DELETE_RESPONSE);
	test_get_response(&server, "http://localhost/foo/state_test", STATE_TEST_RESPONSE);
}
