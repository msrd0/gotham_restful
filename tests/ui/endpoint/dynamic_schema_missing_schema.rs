#[macro_use]
extern crate gotham_restful;

use gotham::hyper::StatusCode;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

fn status_codes() -> Vec<StatusCode> {
	unimplemented!()
}

#[read_all(status_codes = "status_codes")]
async fn read_all() {}

fn main() {}
