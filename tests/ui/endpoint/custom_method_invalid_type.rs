#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[endpoint(method = "String::new()", uri = "custom_read")]
async fn read_all() {}

fn main() {}
