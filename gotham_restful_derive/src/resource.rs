use crate::{
	method::Method,
	util::CollectToResult
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream},
	punctuated::Punctuated,
	token::Comma,
	Error,
	Ident,
	ItemStruct,
	parenthesized,
	parse_macro_input
};
use std::{iter, str::FromStr};

struct MethodList(Punctuated<Ident, Comma>);

impl Parse for MethodList
{
	fn parse(input: ParseStream) -> Result<Self, Error>
	{
		let content;
		let _paren = parenthesized!(content in input);
		let list : Punctuated<Ident, Comma> = Punctuated::parse_separated_nonempty(&content)?;
		Ok(Self(list))
	}
}

fn expand(tokens : TokenStream) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let input = parse_macro_input::parse::<ItemStruct>(tokens)?;
	let ident = input.ident;
	let name = ident.to_string();
	
	let methods = input.attrs.into_iter().filter(|attr|
		attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("rest_resource".to_string()) // TODO wtf
	).map(|attr| {
		syn::parse2(attr.tokens).map(|m : MethodList| m.0.into_iter())
	}).flat_map(|list| match list {
		Ok(iter) => Box::new(iter.map(|method| {
			let method = Method::from_str(&method.to_string()).map_err(|err| Error::new(method.span(), err))?;
			let mod_ident = method.mod_ident(&name);
			let ident = method.setup_ident(&name);
			Ok(quote!(#mod_ident::#ident(&mut route);))
		})) as Box<dyn Iterator<Item = Result<TokenStream2, Error>>>,
		Err(err) => Box::new(iter::once(Err(err)))
	}).collect_to_result()?;
	
	Ok(quote! {
		impl #krate::Resource for #ident
		{
			fn name() -> String
			{
				stringify!(#ident).to_string()
			}
			
			fn setup<D : #krate::DrawResourceRoutes>(mut route : D)
			{
				#(#methods)*
			}
		}
	})
}

pub fn expand_resource(tokens : TokenStream) -> TokenStream
{
	expand(tokens)
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}
