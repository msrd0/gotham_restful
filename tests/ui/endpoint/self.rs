#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all]
fn read_all(self) {}

fn main() {}
