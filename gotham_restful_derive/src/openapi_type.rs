use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	Field,
	Fields,
	ItemStruct,
	parse_macro_input
};

fn expand_field(field : &Field) -> TokenStream2
{
	let ident = match &field.ident {
		Some(ident) => ident,
		None => panic!("Fields without ident are not supported")
	};
	let ty = &field.ty;
	
	quote! {{
		let mut schema = <#ty>::to_schema();
		
		if schema.nullable
		{
			schema.nullable = false;
			schema.name = schema.name.map(|name|
				if name.ends_with("OrNull") { name[..(name.len()-6)].to_string() } else { name });
		}
		else
		{
			required.push(stringify!(#ident).to_string());
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
					ReferenceOr::Item(Box::new(<#ty>::to_schema().to_schema()))
				);
			}
		}
	}}
}

pub fn expand(tokens : proc_macro::TokenStream) -> TokenStream
{
	let input = parse_macro_input!(tokens as ItemStruct);
	
	let ident = input.ident;
	let generics = input.generics;
	
	let fields : Vec<TokenStream2> = match input.fields {
		Fields::Named(fields) => {
			fields.named.iter().map(|field| expand_field(field)).collect()
		},
		Fields::Unnamed(_) => panic!("Unnamed fields are not supported"),
		Fields::Unit => Vec::new()
	};
	
	let output = quote!{
		impl #generics ::gotham_restful::OpenapiType for #ident #generics
		{
			fn to_schema() -> ::gotham_restful::OpenapiSchema
			{
				use ::gotham_restful::{helper::openapi::*, OpenapiSchema};
				
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
	};
	
	eprintln!("output: {}", output);
	output.into()
}
