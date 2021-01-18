#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all(method = "gotham_restful::gotham::hyper::Method::GET")]
async fn read_all() {}

fn main() {}
