#[macro_use] extern crate gotham_restful;

#[derive(OpenapiType)]
struct Foo
{
	#[openapi(nullable = "yes, please")]
	bar : String
}

fn main()
{
}
