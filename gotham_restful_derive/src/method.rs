use heck::SnakeCase;
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
	
	pub fn setup_ident(&self, resource : String) -> Ident
	{
		format_ident!("{}_{}_setup_impl", resource.to_snake_case(), self.fn_ident())
	}
}

pub fn expand_method(method : Method, attrs : TokenStream, item : TokenStream) -> TokenStream
{
	let krate = super::krate();
	let resource_ident = parse_macro_input!(attrs as Ident);
	let fun = parse_macro_input!(item as ItemFn);
	let fun_ident = &fun.sig.ident;
	let fun_vis = &fun.vis;
	
	let trait_ident = method.trait_ident();
	let method_ident = method.fn_ident();
	let setup_ident = method.setup_ident(resource_ident.to_string());
	
	let (ret, is_no_content) = match &fun.sig.output {
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
	let args_state = args.iter().map(|(pat, _)| pat).nth(0).expect("state parameter is required");
	let args_conn = if cfg!(feature = "database") {
		args.iter().filter(|(pat, _)| pat.to_string() == "conn").nth(0)
	} else { None };
	let mut generics : Vec<TokenStream2> = args.iter().skip(1)
		.filter(|(pat, _)| Some(pat.to_string()) != args_conn.map(|(pat, _)| pat.to_string()))
		.map(|(_, ty)| quote!(#ty)).collect();
	generics.push(quote!(#ret));
	let args_def : Vec<TokenStream2> = args.iter()
		.filter(|(pat, _)| Some(pat.to_string()) != args_conn.map(|(pat, _)| pat.to_string()))
		.map(|(pat, ty)| quote!(#pat : #ty)).collect();
	let args_pass : Vec<TokenStream2> = args.iter().map(|(pat, _)| quote!(#pat)).collect();
	let mut block = if is_no_content { quote!(#fun_ident(#(#args_pass),*); Default::default()) } else { quote!(#fun_ident(#(#args_pass),*)) };
	if /*cfg!(feature = "database") &&*/ let Some((conn_pat, conn_ty)) = args_conn // https://github.com/rust-lang/rust/issues/53667
	{
		let repo_ident = format_ident!("{}_database_repo", conn_pat.to_string());
		block = quote! {
			let #repo_ident = <#krate::export::Repo<#conn_ty>>::borrow_from(&#args_state).clone();
			#repo_ident.run(move |#conn_pat| {
				#block
			}).wait()
		};
	}
	
	let output = quote! {
		#fun
		
		impl #krate::#trait_ident<#(#generics),*> for #resource_ident
		where #resource_ident : #krate::Resource
		{
			fn #method_ident(#(#args_def),*) -> #ret
			{
				#block
			}
		}
		
		#[deny(dead_code)]
		#fun_vis fn #setup_ident<D : #krate::DrawResourceRoutes>(route : &mut D)
		{
			route.#method_ident::<#resource_ident, #(#generics),*>();
		}
	};
	output.into()
}
