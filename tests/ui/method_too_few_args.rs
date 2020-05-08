#[macro_use] extern crate gotham_restful;

#[derive(Resource)]
#[resource(read)]
struct FooResource;

#[read(FooResource)]
fn read()
{
}

fn main()
{
}
