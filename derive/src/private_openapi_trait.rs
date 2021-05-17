use crate::util::{remove_parens, CollectToResult};
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
	parse::Parse, spanned::Spanned, Attribute, AttributeArgs, Error, ItemTrait, LitStr, Meta, NestedMeta, PredicateType,
	Result, TraitItem, WherePredicate
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
			if attr.path.is_ident("openapi_only") {
				openapi_only = true;
			} else if attr.path.is_ident("openapi_bound") {
				let attr_arg: LitStr = syn::parse2(remove_parens(attr.tokens))?;
				let predicate = attr_arg.parse_with(WherePredicate::parse)?;
				openapi_bound.push(match predicate {
					WherePredicate::Type(ty) => ty,
					_ => return Err(Error::new(predicate.span(), "Expected type bound"))
				});
			} else if attr.path.is_ident("non_openapi_bound") {
				let attr_arg: LitStr = syn::parse2(remove_parens(attr.tokens))?;
				let predicate = attr_arg.parse_with(WherePredicate::parse)?;
				non_openapi_bound.push(match predicate {
					WherePredicate::Type(ty) => ty,
					_ => return Err(Error::new(predicate.span(), "Expected type bound"))
				});
			} else {
				other.push(attr);
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

pub(crate) fn expand_private_openapi_trait(mut attrs: AttributeArgs, tr8: ItemTrait) -> Result<TokenStream> {
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
	let openapi_ident = match attrs.remove(0) {
		NestedMeta::Meta(Meta::Path(path)) => path,
		p => {
			return Err(Error::new(
				p.span(),
				"Expected name of the Resource struct this method belongs to"
			))
		},
	};

	let orig_trait = {
		let items = tr8
			.items
			.clone()
			.into_iter()
			.map(|item| {
				Ok(match item {
					TraitItem::Method(mut method) => {
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
								.filter(|param| param.ident.to_string() == bound.bounded_ty.to_token_stream().to_string())
								.for_each(|param| param.bounds.extend(bound.bounds.clone()));
						}
						if attrs.openapi_only {
							None
						} else {
							Some(TraitItem::Method(method))
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
					TraitItem::Method(mut method) => {
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
								.filter(|param| param.ident.to_string() == bound.bounded_ty.to_token_stream().to_string())
								.for_each(|param| param.bounds.extend(bound.bounds.clone()));
						}
						TraitItem::Method(method)
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
