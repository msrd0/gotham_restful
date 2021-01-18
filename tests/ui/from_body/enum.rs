#[macro_use]
extern crate gotham_restful;

#[derive(FromBody)]
enum FromBodyEnum {
	SomeVariant(Vec<u8>),
	OtherVariant(String)
}

fn main() {}
