use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	parse::{Parse, ParseStream, Result as SynResult},
	punctuated::Punctuated,
	token::Comma,
	Error,
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
					..Default::default()
				})))
			}
		}
	}
}

fn expand(tokens : TokenStream) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let input = parse_macro_input::parse::<ItemStruct>(tokens)?;
	let ident = input.ident;
	let generics = input.generics;
	
	let mut types : Vec<Path> = Vec::new();
	let mut errors : Vec<Error> = Vec::new();
	for attr in input.attrs.into_iter().filter(|attr|
		attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("supported_types".to_string()) // TODO wtf
	) {
		match syn::parse2::<MimeList>(attr.tokens) {
			Ok(m) => types.extend(m.0),
			Err(e) => errors.push(e)
		}
	}
	if !errors.is_empty()
	{
		let mut iter = errors.into_iter();
		let first = iter.nth(0).unwrap();
		return Err(iter.fold(first, |mut e0, e1| { e0.combine(e1); e0 }));
	}
	
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
