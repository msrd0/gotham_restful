use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream, Result as SynResult},
	punctuated::Punctuated,
	token::Comma,
	Generics,
	Ident,
	ItemStruct,
	Path,
	parenthesized,
	parse_macro_input
};

struct MimeList(Punctuated<Path, Comma>);

impl Parse for MimeList
{
	fn parse(input: ParseStream) -> SynResult<Self>
	{
		let content;
		let _paren = parenthesized!(content in input);
		let list : Punctuated<Path, Comma> = Punctuated::parse_separated_nonempty(&content)?;
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
					pattern: None,
					enumeration: Vec::new()
				})))
			}
		}
	}
}

pub fn expand_request_body(tokens : TokenStream) -> TokenStream
{
	let krate = super::krate();
	let input = parse_macro_input!(tokens as ItemStruct);
	let ident = input.ident;
	let generics = input.generics;
	
	let types : Vec<Path> = input.attrs.into_iter().filter(|attr|
		attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("supported_types".to_string()) // TODO wtf
	).flat_map(|attr| {
		let m : MimeList = syn::parse2(attr.tokens).expect("unable to parse attributes");
		m.0.into_iter()
	}).collect();
	
	let types = match types {
		ref types if types.is_empty() => quote!(None),
		types => quote!(Some(vec![#(#types),*]))
	};
	
	let impl_openapi_type = impl_openapi_type(&ident, &generics);
	
	let output = quote! {
		impl #generics #krate::RequestBody for #ident #generics
		where #ident #generics : #krate::FromBody
		{
			fn supported_types() -> Option<Vec<#krate::Mime>>
			{
				#types
			}
		}
		
		#impl_openapi_type
	};
	output.into()
}
