#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read)]
struct FooResource;

#[read]
fn read() {}

fn main() {}
