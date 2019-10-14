use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
	FnArg,
	ItemFn,
	ReturnType,
	parse_macro_input
};
use std::str::FromStr;

pub enum Method
{
	ReadAll,
	Read,
	Search,
	Create,
	UpdateAll,
	Update,
	DeleteAll,
	Delete
}

impl FromStr for Method
{
	type Err = String;
	fn from_str(str : &str) -> Result<Self, Self::Err>
	{
		match str {
			"ReadAll" | "read_all" => Ok(Self::ReadAll),
			"Read" | "read" => Ok(Self::Read),
			"Search" | "search" => Ok(Self::Search),
			"Create" | "create" => Ok(Self::Create),
			"UpdateAll" | "update_all" => Ok(Self::UpdateAll),
			"Update" | "update" => Ok(Self::Update),
			"DeleteAll" | "delete_all" => Ok(Self::DeleteAll),
			"Delete" | "delete" => Ok(Self::Delete),
			_ => Err("unknown method".to_string())
		}
	}
}

impl Method
{
	pub fn trait_ident(&self) -> Ident
	{
		use Method::*;
		
		let name = match self {
			ReadAll => "ReadAll",
			Read => "Read",
			Search => "Search",
			Create => "Create",
			UpdateAll => "UpdateAll",
			Update => "Update",
			DeleteAll => "DeleteAll",
			Delete => "Delete"
		};
		format_ident!("Resource{}", name)
	}
	
	pub fn fn_ident(&self) -> Ident
	{
		use Method::*;
		
		let name = match self {
			ReadAll => "read_all",
			Read => "read",
			Search => "search",
			Create => "create",
			UpdateAll => "update_all",
			Update => "update",
			DeleteAll => "delete_all",
			Delete => "delete"
		};
		format_ident!("{}", name)
	}
	
	pub fn setup_ident(&self) -> Ident
	{
		format_ident!("{}_setup_impl", self.fn_ident())
	}
}

pub fn expand_method(method : Method, attrs : TokenStream, item : TokenStream) -> TokenStream
{
	let krate = super::krate();
	let ident = parse_macro_input!(attrs as Ident);
	let fun = parse_macro_input!(item as ItemFn);
	
	let (ret, is_no_content) = match fun.sig.output {
		ReturnType::Default => (quote!(#krate::NoContent), true),
		ReturnType::Type(_, ty) => (quote!(#ty), false)
	};
	let args : Vec<(TokenStream2, TokenStream2)> = fun.sig.inputs.iter().map(|arg| match arg {
		FnArg::Typed(arg) => {
			let pat = &arg.pat;
			let ty = &arg.ty;
			(quote!(#pat), quote!(#ty))
		},
		FnArg::Receiver(_) => panic!("didn't expect self parameter")
	}).collect();
	let mut generics : Vec<TokenStream2> = args.iter().skip(1).map(|(_, ty)| quote!(#ty)).collect();
	generics.push(quote!(#ret));
	let args : Vec<TokenStream2> = args.into_iter().map(|(pat, ty)| quote!(#pat : #ty)).collect();
	let block = fun.block.stmts;
	let ret_stmt = if is_no_content { Some(quote!(Default::default())) } else { None };
	
	let trait_ident = method.trait_ident();
	let fn_ident = method.fn_ident();
	let setup_ident = method.setup_ident();
	
	let output = quote! {
		impl #krate::#trait_ident<#(#generics),*> for #ident
		where #ident : #krate::Resource
		{
			fn #fn_ident(#(#args),*) -> #ret
			{
				#(#block)*
				#ret_stmt
			}
		}
		
		#[deny(dead_code)]
		fn #setup_ident<D : #krate::DrawResourceRoutes>(route : &mut D)
		{
			route.#fn_ident::<#ident, #(#generics),*>();
		}
	};
	output.into()
}
