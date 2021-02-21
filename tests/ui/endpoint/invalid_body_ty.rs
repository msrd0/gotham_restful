#[macro_use]
extern crate gotham_restful;
use gotham_restful::gotham::hyper::Method;

#[derive(Resource)]
#[resource(endpoint)]
struct FooResource;

#[derive(Debug)]
struct FooBody {
	foo: String
}

#[endpoint(method = "Method::GET", uri = "", body = true)]
fn endpoint(_: FooBody) {
	unimplemented!()
}

fn main() {}
