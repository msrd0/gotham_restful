#[macro_use]
extern crate gotham_restful;

#[derive(Resource)]
#[resource(read_all)]
struct FooResource;

fn schema(_: u16) -> String {
	unimplemented!()
}

fn status_codes() -> Vec<u16> {
	unimplemented!()
}

#[read_all(schema = "schema", status_codes = "status_codes")]
async fn read_all() {}

fn main() {}
