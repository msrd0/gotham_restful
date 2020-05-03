use crate::util::CollectToResult;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter;
use syn::{
	parenthesized,
	parse::{Parse, ParseStream, Result as SynResult},
	punctuated::Punctuated,
	DeriveInput,
	Error,
	Generics,
	Ident,
	Path,
	Token
};

struct MimeList(Punctuated<Path, Token![,]>);

impl Parse for MimeList
{
	fn parse(input: ParseStream) -> SynResult<Self>
	{
		let content;
		let _paren = parenthesized!(content in input);
		let list = Punctuated::parse_separated_nonempty(&content)?;
		Ok(Self(list))
	}
}

#[cfg(not(feature = "openapi"))]
fn impl_openapi_type(_ident : &Ident, _generics : &Generics) -> TokenStream2
{
	quote!()
}

#[cfg(feature = "openapi")]
fn impl_openapi_type(ident : &Ident, generics : &Generics) -> TokenStream2
{
	let krate = super::krate();
	
	quote! {
		impl #generics #krate::OpenapiType for #ident #generics
		{
			fn schema() -> #krate::OpenapiSchema
			{
				use #krate::{export::openapi::*, OpenapiSchema};
				
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
					format: VariantOrUnknownOrEmpty::Item(StringFormat::Binary),
					..Default::default()
				})))
			}
		}
	}
}

fn expand(tokens : TokenStream) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let input : DeriveInput = syn::parse(tokens)?;
	let ident = input.ident;
	let generics = input.generics;
	
	let types = input.attrs.into_iter()
		.filter(|attr| attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("supported_types".to_string()))
		.flat_map(|attr|
			syn::parse2::<MimeList>(attr.tokens)
				.map(|list| Box::new(list.0.into_iter().map(Ok)) as Box<dyn Iterator<Item = Result<Path, Error>>>)
				.unwrap_or_else(|err| Box::new(iter::once(Err(err)))))
		.collect_to_result()?;
	
	let types = match types {
		ref types if types.is_empty() => quote!(None),
		types => quote!(Some(vec![#(#types),*]))
	};
	
	let impl_openapi_type = impl_openapi_type(&ident, &generics);
	
	Ok(quote! {
		impl #generics #krate::RequestBody for #ident #generics
		where #ident #generics : #krate::FromBody
		{
			fn supported_types() -> Option<Vec<#krate::Mime>>
			{
				#types
			}
		}
		
		#impl_openapi_type
	})
}

pub fn expand_request_body(tokens : TokenStream) -> TokenStream
{
	expand(tokens)
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}
