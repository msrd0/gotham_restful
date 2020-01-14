use heck::SnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
	FnArg,
	ItemFn,
	ReturnType,
	Type,
	TypePath,
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
	
	// extract arguments into pattern, ident and type
	let state_ident = format_ident!("state");
	let args : Vec<(usize, TokenStream2, Ident, Type)> = fun.sig.inputs.iter().enumerate().map(|(i, arg)| match arg {
		FnArg::Typed(arg) => {
			let pat = &arg.pat;
			let ident = format_ident!("arg{}", i);
			(i, quote!(#pat), ident, *arg.ty.clone())
		},
		FnArg::Receiver(_) => panic!("didn't expect self parameter")
	}).collect();
	
	// find the database connection if enabled and present
	let repo_ident = format_ident!("database_repo");
	let args_conn = if cfg!(feature = "database") {
		args.iter().filter(|(_, pat, _, _)| pat.to_string() == "conn").nth(0)
	} else { None };
	let args_conn_name = args_conn.map(|(_, pat, _, _)| pat.to_string());
	
	// extract the generic parameters to use
	let mut generics : Vec<TokenStream2> = args.iter().skip(1)
		.filter(|(_, pat, _, _)| Some(pat.to_string()) != args_conn_name)
		.map(|(_, _, _, ty)| quote!(#ty)).collect();
	generics.push(quote!(#ret));
	
	// extract the definition of our method
	let mut args_def : Vec<TokenStream2> = args.iter()
		.filter(|(_, pat, _, _)| Some(pat.to_string()) != args_conn_name)
		.map(|(i, _, ident, ty)| if *i == 0 { quote!(#state_ident : #ty) } else { quote!(#ident : #ty) }).collect();
	if let Some(_) = args_conn
	{
		args_def.insert(0, quote!(#state_ident : &mut #krate::export::State));
	}
	
	// extract the arguments to pass over to the supplied method
	let args_pass : Vec<TokenStream2> = args.iter().map(|(i, pat, ident, _)| if Some(pat.to_string()) != args_conn_name {
		if *i == 0 { quote!(#state_ident) } else { quote!(#ident) }
	} else {
		quote!(&#ident)
	}).collect();
	
	// prepare the method block
	let mut block = if is_no_content { quote!(#fun_ident(#(#args_pass),*); Default::default()) } else { quote!(#fun_ident(#(#args_pass),*)) };
	if /*cfg!(feature = "database") &&*/ let Some((_, _, conn_ident, conn_ty)) = args_conn // https://github.com/rust-lang/rust/issues/53667
	{
		let conn_ty_real = match conn_ty {
			Type::Reference(ty) => &*ty.elem,
			ty => ty
		};
		block = quote! {
			use #krate::export::{Future, FromState};
			let #repo_ident = <#krate::export::Repo<#conn_ty_real>>::borrow_from(&#state_ident).clone();
			#repo_ident.run::<_, #ret, ()>(move |#conn_ident| {
				Ok(#block)
			}).wait().unwrap()
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
