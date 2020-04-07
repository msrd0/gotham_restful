use crate::method::Method;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream, Result as SynResult},
	punctuated::Punctuated,
	token::Comma,
	Ident,
	ItemStruct,
	parenthesized,
	parse_macro_input
};
use std::str::FromStr;

struct MethodList(Punctuated<Ident, Comma>);

impl Parse for MethodList
{
	fn parse(input: ParseStream) -> SynResult<Self>
	{
		let content;
		let _paren = parenthesized!(content in input);
		let list : Punctuated<Ident, Comma> = Punctuated::parse_separated_nonempty(&content)?;
		Ok(Self(list))
	}
}

pub fn expand_resource(tokens : TokenStream) -> TokenStream
{
	let krate = super::krate();
	let input = parse_macro_input!(tokens as ItemStruct);
	let ident = input.ident;
	let name = ident.to_string();
	
	let methods : Vec<TokenStream2> = input.attrs.into_iter().filter(|attr|
		attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("rest_resource".to_string()) // TODO wtf
	).flat_map(|attr| {
		let m : MethodList = syn::parse2(attr.tokens).expect("unable to parse attributes");
		m.0.into_iter()
	}).map(|method| {
		let method = Method::from_str(&method.to_string()).expect("unknown method");
		let mod_ident = method.mod_ident(&name);
		let ident = method.setup_ident(&name);
		quote!(#mod_ident::#ident(&mut route);)
	}).collect();
	
	let output = quote! {
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
	};
	output.into()
}
