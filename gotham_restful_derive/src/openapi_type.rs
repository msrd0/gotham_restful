use proc_macro::TokenStream;
use proc_macro2::{
	Delimiter,
	TokenStream as TokenStream2,
	TokenTree
};
use quote::quote;
use std::{iter, iter::FromIterator};
use syn::{
	Attribute,
	AttributeArgs,
	Field,
	Fields,
	Generics,
	GenericParam,
	Item,
	ItemEnum,
	ItemStruct,
	Lit,
	Meta,
	NestedMeta,
	Variant,
	parse_macro_input
};

pub fn expand(tokens : TokenStream) -> TokenStream
{
	let input = parse_macro_input!(tokens as Item);
	
	let output = match input {
		Item::Enum(item) => expand_enum(item),
		Item::Struct(item) => expand_struct(item),
		_ => panic!("derive(OpenapiType) not supported for this context")
	};
	output.into()
}

fn expand_where(generics : &Generics) -> TokenStream2
{
	if generics.params.is_empty()
	{
		quote!()
	}
	else
	{
		let krate = super::krate();
		let idents = generics.params.iter()
			.map(|param| match param {
				GenericParam::Type(ty) => Some(ty.ident.clone()),
				_ => None
			})
			.filter(|param| param.is_some())
			.map(|param| param.unwrap());
		
		quote! {
			where #(#idents : #krate::OpenapiType),*
		}
	}
}

#[derive(Debug, Default)]
struct Attrs
{
	nullable : bool,
	rename : Option<String>
}

fn to_string(lit : &Lit) -> String
{
	match lit {
		Lit::Str(str) => str.value(),
		_ => panic!("Expected str, found {}", quote!(#lit))
	}
}

fn to_bool(lit : &Lit) -> bool
{
	match lit {
		Lit::Bool(bool) => bool.value,
		_ => panic!("Expected bool,  found {}", quote!(#lit))
	}
}

fn remove_parens(input : TokenStream2) -> TokenStream2
{
	let iter = input.into_iter().flat_map(|tt| {
		if let TokenTree::Group(group) = &tt
		{
			if group.delimiter() == Delimiter::Parenthesis
			{
				return Box::new(group.stream().into_iter()) as Box<dyn Iterator<Item = TokenTree>>;
			}
		}
		Box::new(iter::once(tt))
	});
	let output = TokenStream2::from_iter(iter);
	output
}

fn parse_attributes(input : &[Attribute]) -> Result<Attrs, syn::Error>
{
	let mut parsed = Attrs::default();
	for attr in input
	{
		if attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("openapi".to_owned())
		{
			let tokens = remove_parens(attr.tokens.clone());
			let nested = parse_macro_input::parse::<AttributeArgs>(tokens.into())?;
			for meta in nested
			{
				match &meta {
					NestedMeta::Meta(Meta::NameValue(kv)) => match kv.path.segments.last().map(|s| s.ident.to_string()) {
						Some(key) => match key.as_ref() {
							"nullable" => parsed.nullable = to_bool(&kv.lit),
						 	"rename" => parsed.rename = Some(to_string(&kv.lit)),
							_ => panic!("Unexpected key: {}", key),
						},
						_ => panic!("Unexpected token: {}", quote!(#meta))
					},
					_ => panic!("Unexpected token: {}", quote!(#meta))
				}
			}
		}
	}
	Ok(parsed)
}

fn expand_variant(variant : &Variant) -> TokenStream2
{
	if variant.fields != Fields::Unit
	{
		panic!("Enum Variants with Fields not supported");
	}
	
	let ident = &variant.ident;
	
	let attrs = parse_attributes(&variant.attrs).expect("Unable to parse attributes");
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	quote! {
		enumeration.push(#name.to_string());
	}
}

fn expand_enum(input : ItemEnum) -> TokenStream2
{
	let krate = super::krate();
	let ident = input.ident;
	let generics = input.generics;
	let where_clause = expand_where(&generics);
	
	let attrs = parse_attributes(&input.attrs).expect("Unable to parse attributes");
	let nullable = attrs.nullable;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	let variants : Vec<TokenStream2> = input.variants.iter().map(expand_variant).collect();
	
	quote! {
		impl #generics #krate::OpenapiType for #ident #generics
		#where_clause
		{
			fn schema() -> #krate::OpenapiSchema
			{
				use #krate::{export::openapi::*, OpenapiSchema};
				
				let mut enumeration : Vec<String> = Vec::new();
				
				#(#variants)*
				
				let schema = SchemaKind::Type(Type::String(StringType {
					format: VariantOrUnknownOrEmpty::Empty,
					enumeration,
					..Default::default()
				}));

				OpenapiSchema {
					name: Some(#name.to_string()),
					nullable: #nullable,
					schema,
					dependencies: Default::default()
				}
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
	
	let attrs = parse_attributes(&field.attrs).expect("Unable to parse attributes");
	let nullable = attrs.nullable;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	quote! {{
		let mut schema = <#ty>::schema();
		
		if schema.nullable
		{
			schema.nullable = false;
		}
		else if !#nullable
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
			Some(schema_name) => {
				properties.insert(
					#name.to_string(),
					ReferenceOr::Reference { reference: format!("#/components/schemas/{}", schema_name) }
				);
				dependencies.insert(schema_name, schema);
			},
			None => {
				properties.insert(
					#name.to_string(),
					ReferenceOr::Item(Box::new(schema.into_schema()))
				);
			}
		}
	}}
}

pub fn expand_struct(input : ItemStruct) -> TokenStream2
{
	let krate = super::krate();
	let ident = input.ident;
	let generics = input.generics;
	let where_clause = expand_where(&generics);
	
	let attrs = parse_attributes(&input.attrs).expect("Unable to parse attributes");
	let nullable = attrs.nullable;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	let fields : Vec<TokenStream2> = match input.fields {
		Fields::Named(fields) => {
			fields.named.iter().map(|field| expand_field(field)).collect()
		},
		Fields::Unnamed(_) => panic!("Unnamed fields are not supported"),
		Fields::Unit => Vec::new()
	};
	
	quote!{
		impl #generics #krate::OpenapiType for #ident #generics
		#where_clause
		{
			fn schema() -> #krate::OpenapiSchema
			{
				use #krate::{export::{openapi::*, IndexMap}, OpenapiSchema};
				
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
					name: Some(#name.to_string()),
					nullable: #nullable,
					schema,
					dependencies
				}
			}
		}
	}
}
