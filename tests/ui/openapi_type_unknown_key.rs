#[macro_use] extern crate gotham_restful;

#[derive(OpenapiType)]
struct Foo
{
	#[openapi(like = "pizza")]
	bar : String
}

fn main()
{
}
