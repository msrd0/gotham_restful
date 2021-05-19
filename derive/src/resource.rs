use crate::{endpoint::endpoint_ident, util::CollectToResult};
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::iter;
use syn::{
	parenthesized,
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	DeriveInput, Result, Token
};

struct MethodList(Punctuated<Ident, Token![,]>);

impl Parse for MethodList {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let content;
		let _paren = parenthesized!(content in input);
		let list = Punctuated::parse_separated_nonempty(&content)?;
		Ok(Self(list))
	}
}

pub fn expand_resource(input: DeriveInput) -> Result<TokenStream> {
	let ident = input.ident;

	let methods = input
		.attrs
		.into_iter()
		.filter(|attr| attr.path.is_ident("resource"))
		.map(|attr| syn::parse2(attr.tokens).map(|m: MethodList| m.0.into_iter()))
		.flat_map(|list| match list {
			Ok(iter) => Box::new(iter.map(|method| {
				let ident = endpoint_ident(&method);
				Ok(quote!(route.endpoint::<#ident>();))
			})) as Box<dyn Iterator<Item = Result<TokenStream>>>,
			Err(err) => Box::new(iter::once(Err(err)))
		})
		.collect_to_result()?;

	let non_openapi_impl = quote! {
		impl ::gotham_restful::Resource for #ident {
			fn setup<D: ::gotham_restful::DrawResourceRoutes>(mut route: D) {
				#(#methods)*
			}
		}
	};
	let openapi_impl = if !cfg!(feature = "openapi") {
		None
	} else {
		Some(quote! {
			impl ::gotham_restful::ResourceWithSchema for #ident {
				fn setup<D: ::gotham_restful::DrawResourceRoutesWithSchema>(mut route: D) {
					#(#methods)*
				}
			}
		})
	};
	Ok(quote! {
		#non_openapi_impl
		#openapi_impl
	})
}
