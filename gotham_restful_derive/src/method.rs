use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
	FnArg,
	ItemFn,
	ReturnType,
	parse_macro_input
};

pub enum Method
{
	ReadAll,
	Read,
	Create,
	UpdateAll,
	Update,
	DeleteAll,
	Delete
}

impl Method
{
	fn trait_ident(&self) -> Ident
	{
		use Method::*;
		
		let name = match self {
			ReadAll => "ReadAll",
			Read => "Read",
			Create => "Create",
			UpdateAll => "UpdateAll",
			Update => "Update",
			DeleteAll => "DeleteAll",
			Delete => "Delete"
		};
		format_ident!("Resource{}", name)
	}
	
	fn fn_ident(&self) -> Ident
	{
		use Method::*;
		
		let name = match self {
			ReadAll => "read_all",
			Read => "read",
			Create => "create",
			UpdateAll => "update_all",
			Update => "update",
			DeleteAll => "delete_all",
			Delete => "delete"
		};
		format_ident!("{}", name)
	}
}

pub fn expand_method(method : Method, attrs : TokenStream, item : TokenStream) -> TokenStream
{
	let ident = parse_macro_input!(attrs as Ident);
	let fun = parse_macro_input!(item as ItemFn);
	
	let (ret, is_no_content) = match fun.sig.output {
		ReturnType::Default => (quote!(::gotham_restful::NoContent), true),
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
	let mut generics : Vec<TokenStream2> = Vec::new();
	for i in 1..args.len()
	{
		let (_, ty) = &args[i];
		generics.push(quote!(#ty));
	}
	generics.push(quote!(#ret));
	let args : Vec<TokenStream2> = args.into_iter().map(|(pat, ty)| quote!(#pat : #ty)).collect();
	let block = fun.block.stmts;
	let ret_stmt = match is_no_content {
		true => Some(quote!(().into())),
		false => None
	};
	
	let trait_ident = method.trait_ident();
	let fn_ident = method.fn_ident();
	
	let output = quote! {
		impl ::gotham_restful::#trait_ident<#(#generics),*> for #ident
		where #ident : ::gotham_restful::Resource
		{
			fn #fn_ident(#(#args),*) -> #ret
			{
				#(#block)*
				#ret_stmt
			}
		}
	};
	output.into()
}
