#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read_any)]
struct FooResource;

#[read_all]
fn read_all() {}

fn main() {}
