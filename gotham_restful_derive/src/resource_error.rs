use crate::util::{CollectToResult, remove_parens};
use lazy_static::lazy_static;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use regex::Regex;
use std::iter;
use syn::{
	parse_macro_input,
	spanned::Spanned,
	Attribute,
	Data,
	DeriveInput,
	Error,
	Fields,
	GenericParam,
	Ident,
	LitStr,
	Path,
	PathSegment,
	Type,
	Variant
};


struct ErrorVariantField
{
	attrs : Vec<Attribute>,
	ident : Ident,
	ty : Type
}

struct ErrorVariant
{
	ident : Ident,
	status : Option<Path>,
	is_named : bool,
	fields : Vec<ErrorVariantField>,
	from_ty : Option<(usize, Type)>,
	display : Option<LitStr>
}

fn process_variant(variant : Variant) -> Result<ErrorVariant, Error>
{
	let status = match variant.attrs.iter()
		.find(|attr| attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("status".to_string()))
	{
		Some(attr) => Some(parse_macro_input::parse::<Path>(remove_parens(attr.tokens.clone()).into())?),
		None => None
	};
	
	let mut is_named = false;
	let mut fields = Vec::new();
	match variant.fields {
		Fields::Named(named) => {
			is_named = true;
			for field in named.named
			{
				let span = field.span();
				fields.push(ErrorVariantField {
					attrs: field.attrs,
					ident: field.ident.ok_or_else(|| Error::new(span, "Missing ident for this enum variant field"))?,
					ty: field.ty
				});
			}
		},
		Fields::Unnamed(unnamed) => {
			for (i, field) in unnamed.unnamed.into_iter().enumerate()
			{
				fields.push(ErrorVariantField {
					attrs: field.attrs,
					ident: format_ident!("arg{}", i),
					ty: field.ty
				})
			}
		},
		Fields::Unit => {}
	}
	
	let from_ty = fields.iter()
		.enumerate()
		.find(|(_, field)| field.attrs.iter().any(|attr| attr.path.segments.last().map(|segment| segment.ident.to_string()) == Some("from".to_string())))
		.map(|(i, field)| (i, field.ty.clone()));
	
	let display = match variant.attrs.iter()
		.find(|attr| attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("display".to_string()))
	{
		Some(attr) => Some(parse_macro_input::parse::<LitStr>(remove_parens(attr.tokens.clone()).into())?),
		None => None
	};
	
	Ok(ErrorVariant {
		ident: variant.ident,
		status,
		is_named,
		fields,
		from_ty,
		display
	})
}

fn path_segment(name : &str) -> PathSegment
{
	PathSegment {
		ident: format_ident!("{}", name),
		arguments: Default::default()
	}
}

lazy_static! {
	// TODO this is a really ugly regex that requires at least two characters between captures
	static ref DISPLAY_REGEX : Regex = Regex::new(r"(^|[^\{])\{(?P<param>[^\}]+)\}([^\}]|$)").unwrap();
}

impl ErrorVariant
{
	fn fields_pat(&self) -> TokenStream2
	{
		let mut fields = self.fields.iter().map(|field| &field.ident).peekable();
		if fields.peek().is_none() {
			quote!()
		} else if self.is_named {
			quote!( { #( #fields ),* } )
		} else {
			quote!( ( #( #fields ),* ) )
		}
	}
	
	fn to_display_match_arm(&self, formatter_ident : &Ident, enum_ident : &Ident) -> Result<TokenStream2, Error>
	{
		let ident = &self.ident;
		let display = self.display.as_ref().ok_or_else(|| Error::new(self.ident.span(), "Missing display string for this variant"))?;
		
		// lets find all required format parameters
		let display_str = display.value();
		let params = DISPLAY_REGEX.captures_iter(&display_str)
			.map(|cap| format_ident!("{}{}", if self.is_named { "" } else { "arg" }, cap.name("param").unwrap().as_str()));
		
		let fields_pat = self.fields_pat();
		Ok(quote! {
			#enum_ident::#ident #fields_pat => write!(#formatter_ident, #display #(, #params = #params)*)
		})
	}
	
	fn into_match_arm(self, krate : &TokenStream2, enum_ident : &Ident) -> TokenStream2
	{
		let ident = &self.ident;
		let fields_pat = self.fields_pat();
		let status = self.status.map(|status| {
			// the status might be relative to StatusCode, so let's fix that
			if status.leading_colon.is_none() && status.segments.len() < 2
			{
				let status_ident = status.segments.first().map(|path| path.clone()).unwrap_or_else(|| path_segment("OK"));
				Path {
					leading_colon: Some(Default::default()),
					segments: vec![path_segment("gotham_restful"), path_segment("gotham"), path_segment("hyper"), path_segment("StatusCode"), status_ident].into_iter().collect()
				}
			}
			else { status }
		});
		
		// the response will come directly from the from_ty if present
		let res = match self.from_ty {
			Some((from_index, _)) => {
				let from_field = &self.fields[from_index].ident;
				quote!(#from_field.into_response_error())
			},
			None => quote!(Ok(#krate::Response {
				status: { #status }.into(),
				body: #krate::gotham::hyper::Body::empty(),
				mime: None
			}))
		};
		
		quote! {
			#enum_ident::#ident #fields_pat => #res
		}
	}
	
	fn were(&self) -> Option<TokenStream2>
	{
		match self.from_ty.as_ref() {
			Some((_, ty)) => Some(quote!( #ty : ::std::error::Error )),
			None => None
		}
	}
}

fn expand(tokens : TokenStream) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	let input = parse_macro_input::parse::<DeriveInput>(tokens)?;
	let ident = input.ident;
	let generics = input.generics;
	
	let inum = match input.data {
		Data::Enum(inum) => Ok(inum),
		Data::Struct(strukt) => Err(strukt.struct_token.span()),
		Data::Union(uni) => Err(uni.union_token.span())
	}.map_err(|span| Error::new(span, "#[derive(ResourceError)] only works for enums"))?;
	let variants = inum.variants.into_iter()
		.map(|variant| process_variant(variant))
		.collect_to_result()?;
	
	let display_impl = if variants.iter().any(|v| v.display.is_none()) { None } else {
		let were = generics.params.iter().filter_map(|param| match param {
			GenericParam::Type(ty) => {
				let ident = &ty.ident;
				Some(quote!(#ident : ::std::fmt::Display))
			},
			_ => None
		});
		let formatter_ident = format_ident!("resource_error_display_formatter");
		let match_arms = variants.iter()
			.map(|v| v.to_display_match_arm(&formatter_ident, &ident))
			.collect_to_result()?;
		Some(quote! {
			impl #generics ::std::fmt::Display for #ident #generics
			where #( #were ),*
			{
				fn fmt(&self, #formatter_ident: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result
				{
					match self {
						#( #match_arms ),*
					}
				}
			}
		})
	};
	
	let mut from_impls : Vec<TokenStream2> = Vec::new();
	
	for var in &variants
	{
		let var_ident = &var.ident;
		let (from_index, from_ty) = match var.from_ty.as_ref() {
			Some(f) => f,
			None => continue
		};
		let from_ident = &var.fields[*from_index].ident;
		
		let fields_pat = var.fields_pat();
		let fields_where = var.fields.iter().enumerate()
			.filter(|(i, _)| i != from_index)
			.map(|(_, field)| {
				let ty = &field.ty;
				quote!( #ty : Default )
			})
			.chain(iter::once(quote!( #from_ty : ::std::error::Error )));
		let fields_let = var.fields.iter().enumerate()
			.filter(|(i, _)| i != from_index)
			.map(|(_, field)| {
				let id = &field.ident;
				let ty = &field.ty;
				quote!( let #id : #ty = Default::default(); )
			});
		
		from_impls.push(quote! {
			impl #generics ::std::convert::From<#from_ty> for #ident #generics
			where #( #fields_where ),*
			{
				fn from(#from_ident : #from_ty) -> Self
				{
					#( #fields_let )*
					Self::#var_ident #fields_pat
				}
			}
		});
	}
	
	let were = variants.iter().filter_map(|variant| variant.were()).collect::<Vec<_>>();
	let variants = variants.into_iter().map(|variant| variant.into_match_arm(&krate, &ident));
	
	Ok(quote! {
		#display_impl
		
		impl #generics #krate::IntoResponseError for #ident #generics
		where #( #were ),*
		{
			type Err = #krate::export::serde_json::Error;
			
			fn into_response_error(self) -> Result<#krate::Response, Self::Err>
			{
				match self {
					#( #variants ),*
				}
			}
		}
		
		#( #from_impls )*
	})
}

pub fn expand_resource_error(tokens : TokenStream) -> TokenStream
{
	expand(tokens)
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}
