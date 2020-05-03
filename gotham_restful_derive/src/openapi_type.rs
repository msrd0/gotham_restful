use crate::util::{CollectToResult, remove_parens};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
	spanned::Spanned,
	Attribute,
	AttributeArgs,
	Data,
	DataEnum,
	DataStruct,
	DeriveInput,
	Error,
	Field,
	Fields,
	Generics,
	GenericParam,
	Ident,
	Lit,
	Meta,
	NestedMeta,
	Variant,
	parse_macro_input
};

pub fn expand(tokens : TokenStream) -> TokenStream
{
	let input = parse_macro_input!(tokens as DeriveInput);
	
	let output = match (input.ident, input.generics, input.attrs, input.data) {
		(ident, generics, attrs, Data::Enum(inum)) => expand_enum(ident, generics, attrs, inum),
		(ident, generics, attrs, Data::Struct(strukt)) => expand_struct(ident, generics, attrs, strukt),
		(_, _, _, Data::Union(uni)) => Err(Error::new(uni.union_token.span(), "#[derive(OpenapiType)] only works for structs and enums"))
	};
	
	output
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}

fn expand_where(generics : &Generics) -> TokenStream2
{
	if generics.params.is_empty()
	{
		return quote!();
	}
	
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

#[derive(Debug, Default)]
struct Attrs
{
	nullable : bool,
	rename : Option<String>
}

fn to_string(lit : &Lit) -> Result<String, Error>
{
	match lit {
		Lit::Str(str) => Ok(str.value()),
		_ => Err(Error::new(lit.span(), "Expected string literal"))
	}
}

fn to_bool(lit : &Lit) -> Result<bool, Error>
{
	match lit {
		Lit::Bool(bool) => Ok(bool.value),
		_ => Err(Error::new(lit.span(), "Expected bool"))
	}
}

fn parse_attributes(input : &[Attribute]) -> Result<Attrs, Error>
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
							"nullable" => parsed.nullable = to_bool(&kv.lit)?,
						 	"rename" => parsed.rename = Some(to_string(&kv.lit)?),
							_ => return Err(Error::new(kv.path.span(), "Unknown key")),
						},
						_ => return Err(Error::new(meta.span(), "Unexpected token"))
					},
					_ => return Err(Error::new(meta.span(), "Unexpected token"))
				}
			}
		}
	}
	Ok(parsed)
}

fn expand_variant(variant : &Variant) -> Result<TokenStream2, Error>
{
	if variant.fields != Fields::Unit
	{
		return Err(Error::new(variant.span(), "Enum Variants with Fields not supported"));
	}
	
	let ident = &variant.ident;
	
	let attrs = parse_attributes(&variant.attrs)?;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	Ok(quote! {
		enumeration.push(#name.to_string());
	})
}

fn expand_enum(ident : Ident, generics : Generics, attrs : Vec<Attribute>, input : DataEnum) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let where_clause = expand_where(&generics);
	
	let attrs = parse_attributes(&attrs)?;
	let nullable = attrs.nullable;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	let variants = input.variants.iter()
		.map(expand_variant)
		.collect_to_result()?;
	
	Ok(quote! {
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
	})
}

fn expand_field(field : &Field) -> Result<TokenStream2, Error>
{
	let ident = match &field.ident {
		Some(ident) => ident,
		None => return Err(Error::new(field.span(), "Fields without ident are not supported"))
	};
	let ty = &field.ty;
	
	let attrs = parse_attributes(&field.attrs)?;
	let nullable = attrs.nullable;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	Ok(quote! {{
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
	}})
}

pub fn expand_struct(ident : Ident, generics : Generics, attrs : Vec<Attribute>, input : DataStruct) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let where_clause = expand_where(&generics);
	
	let attrs = parse_attributes(&attrs)?;
	let nullable = attrs.nullable;
	let name = match attrs.rename {
		Some(rename) => rename,
		None => ident.to_string()
	};
	
	let fields : Vec<TokenStream2> = match input.fields {
		Fields::Named(named_fields) => {
			named_fields.named.iter()
				.map(expand_field)
				.collect_to_result()?
		},
		Fields::Unnamed(fields) => return Err(Error::new(fields.span(), "Unnamed fields are not supported")),
		Fields::Unit => Vec::new()
	};
	
	Ok(quote!{
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
	})
}
