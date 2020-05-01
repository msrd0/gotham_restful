use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod util;

mod from_body;
use from_body::expand_from_body;
mod method;
use method::{expand_method, Method};
mod request_body;
use request_body::expand_request_body;
mod resource;
use resource::expand_resource;
mod resource_error;
use resource_error::expand_resource_error;
#[cfg(feature = "openapi")]
mod openapi_type;

#[inline]
fn print_tokens(tokens : TokenStream) -> TokenStream
{
	//eprintln!("{}", tokens);
	tokens
}

fn krate() -> TokenStream2
{
	quote!(::gotham_restful)
}

#[proc_macro_derive(FromBody)]
pub fn derive_from_body(tokens : TokenStream) -> TokenStream
{
	print_tokens(expand_from_body(tokens))
}

#[cfg(feature = "openapi")]
#[proc_macro_derive(OpenapiType, attributes(openapi))]
pub fn derive_openapi_type(tokens : TokenStream) -> TokenStream
{
	print_tokens(openapi_type::expand(tokens))
}

#[proc_macro_derive(RequestBody, attributes(supported_types))]
pub fn derive_request_body(tokens : TokenStream) -> TokenStream
{
	print_tokens(expand_request_body(tokens))
}

#[proc_macro_derive(Resource, attributes(rest_resource))]
pub fn derive_resource(tokens : TokenStream) -> TokenStream
{
	print_tokens(expand_resource(tokens))
}

#[proc_macro_derive(ResourceError, attributes(display, from, status))]
pub fn derive_resource_error(tokens : TokenStream) -> TokenStream
{
	print_tokens(expand_resource_error(tokens))
}


#[proc_macro_attribute]
pub fn rest_read_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::ReadAll, attr, item))
}

#[proc_macro_attribute]
pub fn rest_read(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::Read, attr, item))
}

#[proc_macro_attribute]
pub fn rest_search(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::Search, attr, item))
}

#[proc_macro_attribute]
pub fn rest_create(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::Create, attr, item))
}

#[proc_macro_attribute]
pub fn rest_update_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::UpdateAll, attr, item))
}

#[proc_macro_attribute]
pub fn rest_update(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::Update, attr, item))
}

#[proc_macro_attribute]
pub fn rest_delete_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::DeleteAll, attr, item))
}

#[proc_macro_attribute]
pub fn rest_delete(attr : TokenStream, item : TokenStream) -> TokenStream
{
	print_tokens(expand_method(Method::Delete, attr, item))
}
