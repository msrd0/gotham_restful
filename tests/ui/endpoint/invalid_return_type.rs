#[macro_use]
extern crate gotham_restful;
use gotham_restful::gotham::hyper::Method;

#[derive(Resource)]
#[resource(endpoint)]
struct FooResource;

struct FooResponse;

#[endpoint(method = "Method::GET", uri = "")]
fn endpoint() -> FooResponse {
	unimplemented!()
}

fn main() {}
