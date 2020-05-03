use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::cmp::min;
use syn::{
	spanned::Spanned,
	Data,
	DeriveInput,
	Error,
	Field,
	Fields,
	Ident,
	Type
};

pub fn expand_from_body(tokens : TokenStream) -> TokenStream
{
	expand(tokens)
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}

struct ParsedFields
{
	fields : Vec<(Ident, Type)>,
	named : bool
}

impl ParsedFields
{
	fn from_named<I>(fields : I) -> Result<Self, Error>
	where
		I : Iterator<Item = Field>
	{
		let fields = fields.map(|field| (field.ident.unwrap(), field.ty)).collect();
		Ok(Self { fields, named: true })
	}
	
	fn from_unnamed<I>(fields : I) -> Result<Self, Error>
	where
		I : Iterator<Item = Field>
	{
		let fields = fields.enumerate().map(|(i, field)| (format_ident!("arg{}", i), field.ty)).collect();
		Ok(Self { fields, named: false })
	}
	
	fn from_unit() -> Result<Self, Error>
	{
		Ok(Self { fields: Vec::new(), named: false })
	}
}

fn expand(tokens : TokenStream) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let input : DeriveInput = syn::parse(tokens)?;
	let ident = input.ident;
	let generics = input.generics;
	
	let strukt = match input.data {
		Data::Enum(inum) => Err(inum.enum_token.span()),
		Data::Struct(strukt) => Ok(strukt),
		Data::Union(uni) => Err(uni.union_token.span())
	}.map_err(|span| Error::new(span, "#[derive(FromBody)] only works for enums"))?;
	
	let fields = match strukt.fields {
		Fields::Named(named) => ParsedFields::from_named(named.named.into_iter())?,
		Fields::Unnamed(unnamed) => ParsedFields::from_unnamed(unnamed.unnamed.into_iter())?,
		Fields::Unit => ParsedFields::from_unit()?
	};
	
	let mut where_clause = quote!();
	let mut block = quote!();
	let mut body_ident = format_ident!("_body");
	let mut type_ident = format_ident!("_type");
	
	if let Some(body_field) = fields.fields.get(0)
	{
		body_ident = body_field.0.clone();
		let body_ty = &body_field.1;
		where_clause = quote! {
			#where_clause
			#body_ty : for<'a> From<&'a [u8]>,
		};
		block = quote! {
			#block
			let #body_ident : &[u8] = &#body_ident;
			let #body_ident : #body_ty = #body_ident.into();
		};
	}
	
	if let Some(type_field) = fields.fields.get(1)
	{
		type_ident = type_field.0.clone();
		let type_ty = &type_field.1;
		where_clause = quote! {
			#where_clause
			#type_ty : From<#krate::Mime>,
		};
		block = quote! {
			#block
			let #type_ident : #type_ty = #type_ident.into();
		};
	}
	
	for field in &fields.fields[min(2, fields.fields.len())..]
	{
		let field_ident = &field.0;
		let field_ty = &field.1;
		where_clause = quote! {
			#where_clause
			#field_ty : Default,
		};
		block = quote! {
			#block
			let #field_ident : #field_ty = Default::default();
		};
	}
	
	let field_names : Vec<&Ident> = fields.fields.iter().map(|field| &field.0).collect();
	let ctor = if fields.named {
		quote!(Self { #(#field_names),* })
	} else {
		quote!(Self ( #(#field_names),* ))
	};
	
	Ok(quote! {
		impl #generics #krate::FromBody for #ident #generics
		where #where_clause
		{
			type Err = #krate::FromBodyNoError;
			
			fn from_body(#body_ident : #krate::gotham::hyper::body::Bytes, #type_ident : #krate::Mime) -> Result<Self, #krate::FromBodyNoError>
			{
				#block
				Ok(#ctor)
			}
		}
	})
}
