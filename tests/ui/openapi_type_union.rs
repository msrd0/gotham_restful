#[macro_use] extern crate gotham_restful;

#[derive(OpenapiType)]
union IntOrPointer
{
	int: u64,
	pointer: *mut String
}

fn main()
{
}
