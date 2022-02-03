use crate::util::CollectToResult;
use either::Either;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::iter;
use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	spanned::Spanned,
	DeriveInput, Error, Generics, Path, Result, Token
};

struct MimeList(Punctuated<Path, Token![,]>);

impl Parse for MimeList {
	fn parse(input: ParseStream<'_>) -> Result<Self> {
		let list = Punctuated::parse_separated_nonempty(input)?;
		Ok(Self(list))
	}
}

#[cfg(not(feature = "openapi"))]
fn impl_openapi_type(_ident: &Ident, _generics: &Generics) -> TokenStream {
	quote!()
}

#[cfg(feature = "openapi")]
fn impl_openapi_type(ident: &Ident, generics: &Generics) -> TokenStream {
	quote! {
		impl #generics ::gotham_restful::private::OpenapiType for #ident #generics {
			fn visit_type<V>(visitor: &mut V)
			where
				V: ::gotham_restful::private::Visitor
			{
				visitor.visit_binary();
			}
		}
	}
}

pub fn expand_request_body(input: DeriveInput) -> Result<TokenStream> {
	let ident = input.ident;
	let generics = input.generics;

	let types = input
		.attrs
		.into_iter()
		.filter(|attr| {
			attr.path
				.segments
				.iter()
				.last()
				.map(|segment| segment.ident.to_string())
				== Some("supported_types".to_string())
		})
		.flat_map(|attr| {
			let span = attr.span();
			attr.parse_args::<MimeList>()
				.map(|list| Either::Left(list.0.into_iter().map(Ok)))
				.unwrap_or_else(|mut err| {
					err.combine(Error::new(
						span,
						"Hint: Types list should look like #[supported_types(TEXT_PLAIN, APPLICATION_JSON)]"
					));
					Either::Right(iter::once(Err(err)))
				})
		})
		.collect_to_result()?;

	let types = match types {
		ref types if types.is_empty() => quote!(None),
		types => quote!(Some(vec![#(#types),*]))
	};

	let impl_openapi_type = impl_openapi_type(&ident, &generics);

	Ok(quote! {
		impl #generics ::gotham_restful::RequestBody for #ident #generics
		where #ident #generics : ::gotham_restful::FromBody
		{
			fn supported_types() -> ::core::option::Option<::std::vec::Vec<::gotham_restful::gotham::mime::Mime>> {
				#types
			}
		}

		#impl_openapi_type
	})
}
