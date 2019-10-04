extern crate proc_macro;

use proc_macro::TokenStream;

mod method;
use method::{expand_method, Method};
#[cfg(feature = "openapi")]
mod openapi_type;

#[cfg(feature = "openapi")]
#[proc_macro_derive(OpenapiType)]
pub fn derive_openapi_type(tokens : TokenStream) -> TokenStream
{
	openapi_type::expand(tokens)
}

#[proc_macro_attribute]
pub fn rest_read_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::ReadAll, attr, item);
	output
}

#[proc_macro_attribute]
pub fn rest_read(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::Read, attr, item);
	output
}

#[proc_macro_attribute]
pub fn rest_create(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::Create, attr, item);
	output
}

#[proc_macro_attribute]
pub fn rest_update_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::UpdateAll, attr, item);
	output
}

#[proc_macro_attribute]
pub fn rest_update(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::Update, attr, item);
	output
}

#[proc_macro_attribute]
pub fn rest_delete_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::DeleteAll, attr, item);
	output
}

#[proc_macro_attribute]
pub fn rest_delete(attr : TokenStream, item : TokenStream) -> TokenStream
{
	let output = expand_method(Method::Delete, attr, item);
	output
}
