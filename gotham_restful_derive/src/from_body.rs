use proc_macro::TokenStream;
use quote::quote;
use syn::{
	Fields,
	ItemStruct,
	parse_macro_input
};

pub fn expand_from_body(tokens : TokenStream) -> TokenStream
{
	let krate = super::krate();
	let input = parse_macro_input!(tokens as ItemStruct);
	let ident = input.ident;
	let generics = input.generics;
	
	let (were, body) = match input.fields {
		Fields::Named(named) => {
			let fields = named.named;
			match fields.len() {
				0 => (quote!(), quote!(Self{})),
				1 => {
					let field = fields.first().unwrap();
					let field_ident = field.ident.as_ref().unwrap();
					let field_ty = &field.ty;
					(quote!(where #field_ty : for<'a> From<&'a [u8]>), quote!(Self { #field_ident: body.into() }))
				},
				_ => panic!("FromBody can only be derived for structs with at most one field")
			}
		},
		Fields::Unnamed(unnamed) => {
			let fields = unnamed.unnamed;
			match fields.len() {
				0 => (quote!(), quote!(Self{})),
				1 => {
					let field = fields.first().unwrap();
					let field_ty = &field.ty;
					(quote!(where #field_ty : for<'a> From<&'a [u8]>), quote!(Self(body.into())))
				},
				_ => panic!("FromBody can only be derived for structs with at most one field")
			}
		},
		Fields::Unit => (quote!(), quote!(Self{}))
	};
	
	let output = quote! {
		impl #generics #krate::FromBody for #ident #generics
		#were
		{
			type Err = String;
			
			fn from_body(body : #krate::Chunk, _content_type : #krate::Mime) -> Result<Self, Self::Err>
			{
				let body : &[u8] = &body;
				Ok(#body)
			}
		}
	};
	output.into()
}