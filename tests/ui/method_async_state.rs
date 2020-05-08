#[macro_use] extern crate gotham_restful;
use gotham_restful::State;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

#[read_all(FooResource)]
async fn read_all(state : &State)
{
}

fn main()
{
}
