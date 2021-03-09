#[macro_use]
extern crate gotham_restful;
use gotham_restful::gotham::hyper::Method;

#[derive(Resource)]
#[resource(endpoint)]
struct FooResource;

#[derive(Debug)]
struct FooPlaceholders {
	foo: String
}

#[endpoint(method = "Method::GET", uri = ":foo")]
fn endpoint(_: FooPlaceholders) {
	unimplemented!()
}

fn main() {}
