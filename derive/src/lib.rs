#![warn(missing_debug_implementations, rust_2018_idioms)]
#![deny(broken_intra_doc_links)]
#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, parse_macro_input::ParseMacroInput, DeriveInput, Result};

mod util;

mod endpoint;
use endpoint::{expand_endpoint, EndpointType};

mod from_body;
use from_body::expand_from_body;

mod request_body;
use request_body::expand_request_body;

mod resource;
use resource::expand_resource;

mod resource_error;
use resource_error::expand_resource_error;

#[cfg(feature = "openapi")]
mod openapi_type;
#[cfg(feature = "openapi")]
use openapi_type::expand_openapi_type;

mod private_openapi_trait;
use private_openapi_trait::expand_private_openapi_trait;

#[inline]
fn print_tokens(tokens: TokenStream2) -> TokenStream {
	// eprintln!("{}", tokens);
	tokens.into()
}

#[inline]
fn expand_derive<F>(input: TokenStream, expand: F) -> TokenStream
where
	F: FnOnce(DeriveInput) -> Result<TokenStream2>
{
	print_tokens(expand(parse_macro_input!(input)).unwrap_or_else(|err| err.to_compile_error()))
}

#[inline]
fn expand_macro<F, A, I>(attrs: TokenStream, item: TokenStream, expand: F) -> TokenStream
where
	F: FnOnce(A, I) -> Result<TokenStream2>,
	A: ParseMacroInput,
	I: ParseMacroInput
{
	print_tokens(expand(parse_macro_input!(attrs), parse_macro_input!(item)).unwrap_or_else(|err| err.to_compile_error()))
}

#[inline]
fn krate() -> TokenStream2 {
	quote!(::gotham_restful)
}

#[proc_macro_derive(FromBody)]
pub fn derive_from_body(input: TokenStream) -> TokenStream {
	expand_derive(input, expand_from_body)
}

#[cfg(feature = "openapi")]
#[proc_macro_derive(OpenapiType, attributes(openapi))]
pub fn derive_openapi_type(input: TokenStream) -> TokenStream {
	expand_derive(input, expand_openapi_type)
}

#[proc_macro_derive(RequestBody, attributes(supported_types))]
pub fn derive_request_body(input: TokenStream) -> TokenStream {
	expand_derive(input, expand_request_body)
}

#[proc_macro_derive(Resource, attributes(resource))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
	expand_derive(input, expand_resource)
}

#[proc_macro_derive(ResourceError, attributes(display, from, status))]
pub fn derive_resource_error(input: TokenStream) -> TokenStream {
	expand_derive(input, expand_resource_error)
}

#[proc_macro_attribute]
pub fn read_all(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::ReadAll, attr, item))
}

#[proc_macro_attribute]
pub fn read(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::Read, attr, item))
}

#[proc_macro_attribute]
pub fn search(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::Search, attr, item))
}

#[proc_macro_attribute]
pub fn create(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::Create, attr, item))
}

#[proc_macro_attribute]
pub fn change_all(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::UpdateAll, attr, item))
}

#[proc_macro_attribute]
pub fn change(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::Update, attr, item))
}

#[proc_macro_attribute]
pub fn remove_all(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::DeleteAll, attr, item))
}

#[proc_macro_attribute]
pub fn remove(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, |attr, item| expand_endpoint(EndpointType::Delete, attr, item))
}

/// PRIVATE MACRO - DO NOT USE
#[doc(hidden)]
#[proc_macro_attribute]
pub fn _private_openapi_trait(attr: TokenStream, item: TokenStream) -> TokenStream {
	expand_macro(attr, item, expand_private_openapi_trait)
}
