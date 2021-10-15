#[macro_use]
extern crate gotham_restful;

use gotham::hyper::StatusCode;
use gotham_restful::private::OpenapiSchema;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

fn schema(_: StatusCode) -> OpenapiSchema {
	unimplemented!()
}

#[read_all(schema = "schema")]
async fn read_all() {}

fn main() {}
