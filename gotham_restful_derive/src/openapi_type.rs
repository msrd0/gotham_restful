use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	Field,
	Fields,
	Item,
	ItemEnum,
	ItemStruct,
	Variant,
	parse_macro_input
};

pub fn expand(tokens : TokenStream) -> TokenStream
{
	let input = parse_macro_input!(tokens as Item);
	
	match input {
		Item::Enum(item) => expand_enum(item),
		Item::Struct(item) => expand_struct(item),
		_ => panic!("derive(OpenapiType) not supported for this context")
	}.into()
}

fn expand_variant(variant : &Variant) -> TokenStream2
{
	if variant.fields != Fields::Unit
	{
		panic!("Enum Variants with Fields not supported");
	}
	
	let ident = &variant.ident;
	
	quote! {
		enumeration.push(stringify!(#ident).to_string());
	}
}

fn expand_enum(input : ItemEnum) -> TokenStream2
{
	let ident = input.ident;
	let generics = input.generics;
	
	let variants : Vec<TokenStream2> = input.variants.iter().map(expand_variant).collect();
	
	quote! {
		impl #generics ::gotham_restful::OpenapiType for #ident #generics
		{
			fn to_schema() -> ::gotham_restful::OpenapiSchema
			{
				use ::gotham_restful::{export::openapi::*, OpenapiSchema};
				
				let mut enumeration : Vec<String> = Vec::new();
				
				#(#variants)*
				
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
					format: VariantOrUnknownOrEmpty::Empty,
					pattern: None,
					enumeration
				})))
			}
		}
	}
}

fn expand_field(field : &Field) -> TokenStream2
{
	let ident = match &field.ident {
		Some(ident) => ident,
		None => panic!("Fields without ident are not supported")
	};
	let ty = &field.ty;
	
	quote! {{
		let mut schema = <#ty>::schema();
		
		if schema.nullable
		{
			schema.nullable = false;
		}
		else
		{
			required.push(stringify!(#ident).to_string());
		}
		
		let keys : Vec<String> = schema.dependencies.keys().map(|k| k.to_string()).collect();
		for dep in keys
		{
			let dep_schema = schema.dependencies.swap_remove(&dep);
			if let Some(dep_schema) = dep_schema
			{
				dependencies.insert(dep, dep_schema);
			}
		}
		
		match schema.name.clone() {
			Some(name) => {
				properties.insert(
					stringify!(#ident).to_string(),
					ReferenceOr::Reference { reference: format!("#/components/schemas/{}", name) }
				);
				dependencies.insert(name, schema);
			},
			None => {
				properties.insert(
					stringify!(#ident).to_string(),
					ReferenceOr::Item(Box::new(schema.into_schema()))
				);
			}
		}
	}}
}

pub fn expand_struct(input : ItemStruct) -> TokenStream2
{
	let ident = input.ident;
	let generics = input.generics;
	
	let fields : Vec<TokenStream2> = match input.fields {
		Fields::Named(fields) => {
			fields.named.iter().map(|field| expand_field(field)).collect()
		},
		Fields::Unnamed(_) => panic!("Unnamed fields are not supported"),
		Fields::Unit => Vec::new()
	};
	
	quote!{
		impl #generics ::gotham_restful::OpenapiType for #ident #generics
		{
			fn schema() -> ::gotham_restful::OpenapiSchema
			{
				use ::gotham_restful::{export::{openapi::*, IndexMap}, OpenapiSchema};
				
				let mut properties : IndexMap<String, ReferenceOr<Box<Schema>>> = IndexMap::new();
				let mut required : Vec<String> = Vec::new();
				let mut dependencies : IndexMap<String, OpenapiSchema> = IndexMap::new();
				
				#(#fields)*
				
				let schema = SchemaKind::Type(Type::Object(ObjectType {
					properties,
					required,
					additional_properties: None,
					min_properties: None,
					max_properties: None
				}));

				OpenapiSchema {
					name: Some(stringify!(#ident).to_string()),
					nullable: false,
					schema,
					dependencies
				}
			}
		}
	}
}
