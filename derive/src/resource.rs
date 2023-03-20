use crate::{endpoint::endpoint_ident, util::CollectToResult};
use either::Either;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::iter;
use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	DeriveInput, Meta, Result, Token
};

struct MethodList(Punctuated<Ident, Token![,]>);

impl Parse for MethodList {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		Ok(Self(Punctuated::parse_separated_nonempty(input)?))
	}
}

pub fn expand_resource(input: DeriveInput) -> Result<TokenStream> {
	let ident = input.ident;

	let methods = input
		.attrs
		.into_iter()
		.filter_map(|attr| match attr.meta {
			Meta::List(list) if list.path.is_ident("resource") => Some(list.tokens),
			_ => None
		})
		.map(|tokens| syn::parse2(tokens).map(|m: MethodList| m.0.into_iter()))
		.flat_map(|list| match list {
			Ok(iter) => Either::Left(iter.map(|method| {
				let ident = endpoint_ident(&method);
				Ok(quote!(route.endpoint::<#ident>();))
			})),
			Err(err) => Either::Right(iter::once(Err(err)))
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
