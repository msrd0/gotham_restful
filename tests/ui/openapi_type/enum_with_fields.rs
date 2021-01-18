#[macro_use]
extern crate gotham_restful;

#[derive(OpenapiType)]
enum Food {
	Pasta,
	Pizza { pineapple: bool },
	Rice,
	Other(String)
}

fn main() {}
