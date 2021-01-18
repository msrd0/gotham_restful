#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all(wants_auth = "yes, please")]
async fn read_all() {}

fn main() {}
