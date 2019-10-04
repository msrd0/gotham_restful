extern crate proc_macro;

use proc_macro::TokenStream;

#[cfg(feature = "openapi")]
mod openapi_type;

#[cfg(feature = "openapi")]
#[proc_macro_derive(OpenapiType)]
pub fn derive_openapi_type(tokens : TokenStream) -> TokenStream
{
	openapi_type::expand(tokens)
}
