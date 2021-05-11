use crate::util::{remove_parens, CollectToResult};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::iter;
use syn::{
	spanned::Spanned, Attribute, Data, DeriveInput, Error, Fields, GenericParam, LitStr, Path, PathSegment, Result, Type,
	Variant
};

struct ErrorVariantField {
	attrs: Vec<Attribute>,
	ident: Ident,
	ty: Type
}

struct ErrorVariant {
	ident: Ident,
	status: Option<Path>,
	is_named: bool,
	fields: Vec<ErrorVariantField>,
	from_ty: Option<(usize, Type)>,
	display: Option<LitStr>
}

fn process_variant(variant: Variant) -> Result<ErrorVariant> {
	let status =
		match variant.attrs.iter().find(|attr| {
			attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("status".to_string())
		}) {
			Some(attr) => Some(syn::parse2(remove_parens(attr.tokens.clone()))?),
			None => None
		};

	let mut is_named = false;
	let mut fields = Vec::new();
	match variant.fields {
		Fields::Named(named) => {
			is_named = true;
			for field in named.named {
				let span = field.span();
				fields.push(ErrorVariantField {
					attrs: field.attrs,
					ident: field
						.ident
						.ok_or_else(|| Error::new(span, "Missing ident for this enum variant field"))?,
					ty: field.ty
				});
			}
		},
		Fields::Unnamed(unnamed) => {
			for (i, field) in unnamed.unnamed.into_iter().enumerate() {
				fields.push(ErrorVariantField {
					attrs: field.attrs,
					ident: format_ident!("arg{}", i),
					ty: field.ty
				})
			}
		},
		Fields::Unit => {}
	}

	let from_ty = fields
		.iter()
		.enumerate()
		.find(|(_, field)| {
			field
				.attrs
				.iter()
				.any(|attr| attr.path.segments.last().map(|segment| segment.ident.to_string()) == Some("from".to_string()))
		})
		.map(|(i, field)| (i, field.ty.clone()));

	let display = match variant.attrs.iter().find(|attr| {
		attr.path.segments.iter().last().map(|segment| segment.ident.to_string()) == Some("display".to_string())
	}) {
		Some(attr) => Some(syn::parse2(remove_parens(attr.tokens.clone()))?),
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

fn path_segment(name: &str) -> PathSegment {
	PathSegment {
		ident: format_ident!("{}", name),
		arguments: Default::default()
	}
}

impl ErrorVariant {
	fn fields_pat(&self) -> TokenStream {
		let mut fields = self.fields.iter().map(|field| &field.ident).peekable();
		if fields.peek().is_none() {
			quote!()
		} else if self.is_named {
			quote!( { #( #fields ),* } )
		} else {
			quote!( ( #( #fields ),* ) )
		}
	}

	fn to_display_match_arm(&self, formatter_ident: &Ident, enum_ident: &Ident) -> Result<TokenStream> {
		let ident = &self.ident;
		let display = self
			.display
			.as_ref()
			.ok_or_else(|| Error::new(self.ident.span(), "Missing display string for this variant"))?;

		// lets find all required format parameters
		let display_str = display.value();
		let mut params: Vec<&str> = Vec::new();
		let len = display_str.len();
		let mut start = len;
		let mut iter = display_str.chars().enumerate().peekable();
		while let Some((i, c)) = iter.next() {
			// we found a new opening brace
			if start == len && c == '{' {
				start = i + 1;
			}
			// we found a duplicate opening brace
			else if start == i && c == '{' {
				start = len;
			}
			// we found a closing brace
			else if start < i && c == '}' {
				match iter.peek() {
					Some((_, '}')) => {
						return Err(Error::new(
							display.span(),
							"Error parsing format string: curly braces not allowed inside parameter name"
						))
					},
					_ => params.push(&display_str[start..i])
				};
				start = len;
			}
			// we found a closing brace without content
			else if start == i && c == '}' {
				return Err(Error::new(
					display.span(),
					"Error parsing format string: parameter name must not be empty"
				));
			}
		}
		if start != len {
			return Err(Error::new(
				display.span(),
				"Error parsing format string: Unmatched opening brace"
			));
		}
		let params = params
			.into_iter()
			.map(|name| format_ident!("{}{}", if self.is_named { "" } else { "arg" }, name));

		let fields_pat = self.fields_pat();
		Ok(quote! {
			#enum_ident::#ident #fields_pat => write!(#formatter_ident, #display #(, #params = #params)*)
		})
	}

	fn into_match_arm(self, krate: &TokenStream, enum_ident: &Ident) -> Result<TokenStream> {
		let ident = &self.ident;
		let fields_pat = self.fields_pat();
		let status = self.status.map(|status| {
			// the status might be relative to StatusCode, so let's fix that
			if status.leading_colon.is_none() && status.segments.len() < 2 {
				let status_ident = status.segments.first().cloned().unwrap_or_else(|| path_segment("OK"));
				Path {
					leading_colon: Some(Default::default()),
					segments: vec![
						path_segment("gotham_restful"),
						path_segment("gotham"),
						path_segment("hyper"),
						path_segment("StatusCode"),
						status_ident,
					]
					.into_iter()
					.collect()
				}
			} else {
				status
			}
		});

		// the response will come directly from the from_ty if present
		let res = match (self.from_ty, status) {
			(Some((from_index, _)), None) => {
				let from_field = &self.fields[from_index].ident;
				quote!(#from_field.into_response_error())
			},
			(Some(_), Some(_)) => return Err(Error::new(ident.span(), "When #[from] is used, #[status] must not be used!")),
			(None, Some(status)) => quote!(Ok(#krate::Response::new(
				{ #status }.into(),
				#krate::gotham::hyper::Body::empty(),
				None
			))),
			(None, None) => return Err(Error::new(ident.span(), "Missing #[status(code)] for this variant"))
		};

		Ok(quote! {
			#enum_ident::#ident #fields_pat => #res
		})
	}

	fn were(&self) -> Option<TokenStream> {
		self.from_ty.as_ref().map(|(_, ty)| quote!( #ty : ::std::error::Error ))
	}
}

pub fn expand_resource_error(input: DeriveInput) -> Result<TokenStream> {
	let krate = super::krate();
	let ident = input.ident;
	let generics = input.generics;

	let inum = match input.data {
		Data::Enum(inum) => Ok(inum),
		Data::Struct(strukt) => Err(strukt.struct_token.span()),
		Data::Union(uni) => Err(uni.union_token.span())
	}
	.map_err(|span| Error::new(span, "#[derive(ResourceError)] only works for enums"))?;
	let variants = inum.variants.into_iter().map(process_variant).collect_to_result()?;

	let display_impl = if variants.iter().any(|v| v.display.is_none()) {
		None // TODO issue warning if display is present on some but not all
	} else {
		let were = generics.params.iter().filter_map(|param| match param {
			GenericParam::Type(ty) => {
				let ident = &ty.ident;
				Some(quote!(#ident : ::std::fmt::Display))
			},
			_ => None
		});
		let formatter_ident = format_ident!("resource_error_display_formatter");
		let match_arms = variants
			.iter()
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

	let mut from_impls: Vec<TokenStream> = Vec::new();

	for var in &variants {
		let var_ident = &var.ident;
		let (from_index, from_ty) = match var.from_ty.as_ref() {
			Some(f) => f,
			None => continue
		};
		let from_ident = &var.fields[*from_index].ident;

		let fields_pat = var.fields_pat();
		let fields_where = var
			.fields
			.iter()
			.enumerate()
			.filter(|(i, _)| i != from_index)
			.map(|(_, field)| {
				let ty = &field.ty;
				quote!( #ty : Default )
			})
			.chain(iter::once(quote!( #from_ty : ::std::error::Error )));
		let fields_let = var
			.fields
			.iter()
			.enumerate()
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
	let variants = variants
		.into_iter()
		.map(|variant| variant.into_match_arm(&krate, &ident))
		.collect_to_result()?;

	Ok(quote! {
		#display_impl

		impl #generics #krate::IntoResponseError for #ident #generics
		where #( #were ),*
		{
			type Err = #krate::private::serde_json::Error;

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
