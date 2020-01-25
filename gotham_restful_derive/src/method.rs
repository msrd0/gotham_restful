use heck::SnakeCase;
use proc_macro::TokenStream;
use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
	Attribute,
	FnArg,
	ItemFn,
	PatType,
	ReturnType,
	Type,
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

enum MethodArgumentType
{
	StateRef,
	StateMutRef,
	MethodArg(Type),
	DatabaseConnection(Type),
	AuthStatus(Type),
	AuthStatusRef(Type)
}

impl MethodArgumentType
{
	fn is_method_arg(&self) -> bool
	{
		match self {
			Self::MethodArg(_) => true,
			_ => false,
		}
	}
	
	fn is_database_conn(&self) -> bool
	{
		match self {
			Self::DatabaseConnection(_) => true,
			_ => false
		}
	}
	
	fn is_auth_status(&self) -> bool
	{
		match self {
			Self::AuthStatus(_) | Self::AuthStatusRef(_) => true,
			_ => false
		}
	}
	
	fn quote_ty(&self) -> Option<TokenStream2>
	{
		match self {
			Self::MethodArg(ty) => Some(quote!(#ty)),
			Self::DatabaseConnection(ty) => Some(quote!(#ty)),
			Self::AuthStatus(ty) => Some(quote!(#ty)),
			Self::AuthStatusRef(ty) => Some(quote!(#ty)),
			_ => None
		}
	}
}

struct MethodArgument
{
	ident : Ident,
	ty : MethodArgumentType
}

fn interpret_arg_ty(index : usize, attrs : &[Attribute], name : &str, ty : Type) -> MethodArgumentType
{
	let attr = attrs.into_iter()
		.filter(|arg| arg.path.segments.iter().filter(|path| &path.ident.to_string() == "rest_arg").nth(0).is_some())
		.nth(0)
		.map(|arg| arg.tokens.to_string());
	
	if cfg!(feature = "auth") && (attr.as_deref() == Some("auth") || (attr.is_none() && name == "auth"))
	{
		return match ty {
			Type::Reference(ty) => MethodArgumentType::AuthStatusRef(*ty.elem),
			ty => MethodArgumentType::AuthStatus(ty)
		};
	}
	
	if cfg!(feature = "database") && (attr.as_deref() == Some("connection") || attr.as_deref() == Some("conn") || (attr.is_none() && name == "conn"))
	{
		return MethodArgumentType::DatabaseConnection(match ty {
			Type::Reference(ty) => *ty.elem,
			ty => ty
		});
	}
	
	if index == 0
	{
		return match ty {
			Type::Reference(ty) => if ty.mutability.is_none() { MethodArgumentType::StateRef } else { MethodArgumentType::StateMutRef },
			_ => panic!("The first argument, unless some feature is used, has to be a (mutable) reference to gotham::state::State")
		};
	}
	
	MethodArgumentType::MethodArg(ty)
}

fn interpret_arg(index : usize, arg : &PatType) -> MethodArgument
{
	let pat = &arg.pat;
	let ident = format_ident!("arg{}", index);
	let orig_name = quote!(#pat);
	let ty = interpret_arg_ty(index, &arg.attrs, &orig_name.to_string(), *arg.ty.clone());
	
	MethodArgument { ident, ty }
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
	
	// some default idents we'll need
	let state_ident = format_ident!("state");
	let repo_ident = format_ident!("repo");
	let conn_ident = format_ident!("conn");
	let auth_ident = format_ident!("auth");
	
	// extract arguments into pattern, ident and type
	let args : Vec<MethodArgument> = fun.sig.inputs.iter().enumerate().map(|(i, arg)| match arg {
		FnArg::Typed(arg) => interpret_arg(i, arg),
		FnArg::Receiver(_) => panic!("didn't expect self parameter")
	}).collect();
	
	// extract the generic parameters to use
	let mut generics : Vec<TokenStream2> = args.iter()
		.filter(|arg| (*arg).ty.is_method_arg())
		.map(|arg| arg.ty.quote_ty().unwrap())
		.collect();
	generics.push(quote!(#ret));
	
	// extract the definition of our method
	let mut args_def : Vec<TokenStream2> = args.iter()
		.filter(|arg| (*arg).ty.is_method_arg())
		.map(|arg| {
			let ident = &arg.ident;
			let ty = arg.ty.quote_ty();
			quote!(#ident : #ty)
		}).collect();
	args_def.insert(0, quote!(#state_ident : &mut #krate::export::State));
	
	// extract the arguments to pass over to the supplied method
	let args_pass : Vec<TokenStream2> = args.iter().map(|arg| match (&arg.ty, &arg.ident) {
		(MethodArgumentType::StateRef, _) => quote!(#state_ident),
		(MethodArgumentType::StateMutRef, _) => quote!(#state_ident),
		(MethodArgumentType::MethodArg(_), ident) => quote!(#ident),
		(MethodArgumentType::DatabaseConnection(_), _) => quote!(&#conn_ident),
		(MethodArgumentType::AuthStatus(_), _) => quote!(#auth_ident),
		(MethodArgumentType::AuthStatusRef(_), _) => quote!(&#auth_ident)
	}).collect();
	
	// prepare the method block
	let mut block = quote!(#fun_ident(#(#args_pass),*));
	if is_no_content
	{
		block = quote!(#block; Default::default())
	}
	if let Some(arg) = args.iter().filter(|arg| (*arg).ty.is_database_conn()).nth(0)
	{
		let conn_ty = arg.ty.quote_ty();
		block = quote! {
			let #repo_ident = <#krate::export::Repo<#conn_ty>>::borrow_from(&#state_ident).clone();
			#repo_ident.run::<_, #ret, ()>(move |#conn_ident| {
				Ok({#block})
			}).wait().unwrap()
		};
	}
	if let Some(arg) = args.iter().filter(|arg| (*arg).ty.is_auth_status()).nth(0)
	{
		let auth_ty = arg.ty.quote_ty();
		block = quote! {
			let #auth_ident : #auth_ty = <#auth_ty>::borrow_from(#state_ident).clone();
			#block
		};
	}
	
	// prepare the where clause
	let mut where_clause = quote!(#resource_ident : #krate::Resource,);
	for arg in args.iter().filter(|arg| (*arg).ty.is_auth_status())
	{
		let auth_ty = arg.ty.quote_ty();
		where_clause = quote!(#where_clause #auth_ty : Clone,);
	}
	
	// put everything together
	let output = quote! {
		#fun
		
		impl #krate::#trait_ident<#(#generics),*> for #resource_ident
		where #where_clause
		{
			fn #method_ident(#(#args_def),*) -> #ret
			{
				#[allow(unused_imports)]
				use #krate::export::{Future, FromState};
				
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
