#[macro_use]
extern crate gotham_restful;
use gotham_restful::gotham::hyper::Method;

#[derive(Resource)]
#[resource(endpoint)]
struct FooResource;

#[derive(Debug)]
struct FooParams {
	foo: String
}

#[endpoint(method = "Method::GET", uri = "", params = true)]
fn endpoint(_: FooParams) {
	unimplemented!()
}

fn main() {}
