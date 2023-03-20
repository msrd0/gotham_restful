use crate::{util::CollectToResult, AttributeArgs};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
	spanned::Spanned, Attribute, Error, ItemTrait, Meta, PredicateType, Result, TraitItem,
	WherePredicate
};

struct TraitItemAttrs {
	openapi_only: bool,
	openapi_bound: Vec<PredicateType>,
	non_openapi_bound: Vec<PredicateType>,
	other_attrs: Vec<Attribute>
}

impl TraitItemAttrs {
	fn parse(attrs: Vec<Attribute>) -> Result<Self> {
		let mut openapi_only = false;
		let mut openapi_bound = Vec::new();
		let mut non_openapi_bound = Vec::new();
		let mut other = Vec::new();

		for attr in attrs {
			match attr.meta {
				Meta::Path(path) if path.is_ident("openapi_only") => {
					openapi_only = true;
				},
				Meta::List(list) if list.path.is_ident("openapi_bound") => {
					let predicate: WherePredicate = syn::parse2(list.tokens)?;
					openapi_bound.push(match predicate {
						WherePredicate::Type(ty) => ty,
						_ => return Err(Error::new(predicate.span(), "Expected type bound"))
					});
				},
				Meta::List(list) if list.path.is_ident("non_openapi_bound") => {
					let predicate: WherePredicate = syn::parse2(list.tokens)?;
					non_openapi_bound.push(match predicate {
						WherePredicate::Type(ty) => ty,
						_ => return Err(Error::new(predicate.span(), "Expected type bound"))
					});
				},
				_ => other.push(attr)
			}
		}

		Ok(Self {
			openapi_only,
			openapi_bound,
			non_openapi_bound,
			other_attrs: other
		})
	}
}

pub(crate) fn expand_private_openapi_trait(
	AttributeArgs(attrs): AttributeArgs,
	tr8: ItemTrait
) -> Result<TokenStream> {
	let mut attrs = attrs.into_iter();

	let tr8_attrs = &tr8.attrs;
	let vis = &tr8.vis;
	let ident = &tr8.ident;
	let generics = &tr8.generics;
	let colon_token = &tr8.colon_token;
	let supertraits = &tr8.supertraits;

	if attrs.len() != 1 {
		return Err(Error::new(
			Span::call_site(),
			"Expected one argument. Example: #[_private_openapi_trait(OpenapiTraitName)]"
		));
	}
	let openapi_ident = match attrs.next() {
		Some(Meta::Path(path)) => path,
		Some(p) => {
			return Err(Error::new(
				p.span(),
				"Expected name of the Resource struct this method belongs to"
			));
		},
		None => {
			return Err(Error::new(
				Span::call_site(),
				"Expected name of the Resource struct this method belongs to"
			));
		}
	};

	let orig_trait = {
		let items = tr8
			.items
			.clone()
			.into_iter()
			.map(|item| {
				Ok(match item {
					TraitItem::Fn(mut method) => {
						let attrs = TraitItemAttrs::parse(method.attrs)?;
						method.attrs = attrs.other_attrs;
						for bound in attrs.non_openapi_bound {
							// we compare two incompatible types using their `Display` implementation
							// this triggers a false positive in clippy
							#[cfg_attr(feature = "cargo-clippy", allow(clippy::cmp_owned))]
							method
								.sig
								.generics
								.type_params_mut()
								.filter(|param| {
									param.ident.to_string()
										== bound.bounded_ty.to_token_stream().to_string()
								})
								.for_each(|param| param.bounds.extend(bound.bounds.clone()));
						}
						if attrs.openapi_only {
							None
						} else {
							Some(TraitItem::Fn(method))
						}
					},
					TraitItem::Type(mut ty) => {
						let attrs = TraitItemAttrs::parse(ty.attrs)?;
						ty.attrs = attrs.other_attrs;
						Some(TraitItem::Type(ty))
					},
					item => Some(item)
				})
			})
			.collect_to_result()?;
		quote! {
			#(#tr8_attrs)*
			#vis trait #ident #generics #colon_token #supertraits {
				#(#items)*
			}
		}
	};

	let openapi_trait = if !cfg!(feature = "openapi") {
		None
	} else {
		let items = tr8
			.items
			.clone()
			.into_iter()
			.map(|item| {
				Ok(match item {
					TraitItem::Fn(mut method) => {
						let attrs = TraitItemAttrs::parse(method.attrs)?;
						method.attrs = attrs.other_attrs;
						for bound in attrs.openapi_bound {
							// we compare two incompatible types using their `Display` implementation
							// this triggers a false positive in clippy
							#[cfg_attr(feature = "cargo-clippy", allow(clippy::cmp_owned))]
							method
								.sig
								.generics
								.type_params_mut()
								.filter(|param| {
									param.ident.to_string()
										== bound.bounded_ty.to_token_stream().to_string()
								})
								.for_each(|param| param.bounds.extend(bound.bounds.clone()));
						}
						TraitItem::Fn(method)
					},
					TraitItem::Type(mut ty) => {
						let attrs = TraitItemAttrs::parse(ty.attrs)?;
						ty.attrs = attrs.other_attrs;
						for bound in attrs.openapi_bound {
							ty.bounds.extend(bound.bounds.clone());
						}
						TraitItem::Type(ty)
					},
					item => item
				})
			})
			.collect_to_result()?;
		Some(quote! {
			#(#tr8_attrs)*
			#vis trait #openapi_ident #generics #colon_token #supertraits {
				#(#items)*
			}
		})
	};

	Ok(quote! {
		#orig_trait
		#openapi_trait
	})
}
