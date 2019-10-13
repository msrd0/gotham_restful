extern crate proc_macro;

use proc_macro::TokenStream;

mod method;
use method::{expand_method, Method};
mod resource;
use resource::expand_resource;
#[cfg(feature = "openapi")]
mod openapi_type;

#[cfg(feature = "openapi")]
#[proc_macro_derive(OpenapiType)]
pub fn derive_openapi_type(tokens : TokenStream) -> TokenStream
{
	openapi_type::expand(tokens)
}

#[proc_macro_derive(Resource, attributes(rest_resource))]
pub fn derive_resource(tokens : TokenStream) -> TokenStream
{
	expand_resource(tokens)
}

#[proc_macro_attribute]
pub fn rest_read_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::ReadAll, attr, item)
}

#[proc_macro_attribute]
pub fn rest_read(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::Read, attr, item)
}

#[proc_macro_attribute]
pub fn rest_search(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::Search, attr, item)
}

#[proc_macro_attribute]
pub fn rest_create(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::Create, attr, item)
}

#[proc_macro_attribute]
pub fn rest_update_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::UpdateAll, attr, item)
}

#[proc_macro_attribute]
pub fn rest_update(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::Update, attr, item)
}

#[proc_macro_attribute]
pub fn rest_delete_all(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::DeleteAll, attr, item)
}

#[proc_macro_attribute]
pub fn rest_delete(attr : TokenStream, item : TokenStream) -> TokenStream
{
	expand_method(Method::Delete, attr, item)
}
