#[macro_use] extern crate gotham_restful;

#[derive(OpenapiType)]
struct Foo
{
	#[openapi(rename = 42)]
	bar : String
}

fn main()
{
}
