#[macro_use]
extern crate gotham_restful;
use gotham::state::State;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all]
async fn read_all(state: &State) {}

fn main() {}
