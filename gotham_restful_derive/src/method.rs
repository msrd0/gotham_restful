use crate::util::CollectToResult;
use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::{
	spanned::Spanned,
	Attribute,
	AttributeArgs,
	Error,
	FnArg,
	ItemFn,
	Lit,
	LitBool,
	Meta,
	NestedMeta,
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
			_ => Err(format!("Unknown method: `{}'", str))
		}
	}
}

impl Method
{
	pub fn type_names(&self) -> Vec<&'static str>
	{
		use Method::*;
		
		match self {
			ReadAll => vec![],
			Read => vec!["ID"],
			Search => vec!["Query"],
			Create => vec!["Body"],
			UpdateAll => vec!["Body"],
			Update => vec!["ID", "Body"],
			DeleteAll => vec![],
			Delete => vec!["ID"]
		}
	}
	
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
	
	pub fn mod_ident(&self, resource : &str) -> Ident
	{
		format_ident!("_gotham_restful_resource_{}_method_{}", resource.to_snake_case(), self.fn_ident())
	}
	
	pub fn handler_struct_ident(&self, resource : &str) -> Ident
	{
		format_ident!("{}{}Handler", resource.to_camel_case(), self.trait_ident())
	}
	
	pub fn setup_ident(&self, resource : &str) -> Ident
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
	ident_span : Span,
	ty : MethodArgumentType
}

impl Spanned for MethodArgument
{
	fn span(&self) -> Span
	{
		self.ident_span
	}
}

fn interpret_arg_ty(index : usize, attrs : &[Attribute], name : &str, ty : Type) -> Result<MethodArgumentType, Error>
{
	let attr = attrs.into_iter()
		.filter(|arg| arg.path.segments.iter().filter(|path| &path.ident.to_string() == "rest_arg").nth(0).is_some())
		.nth(0)
		.map(|arg| arg.tokens.to_string());
	
	if cfg!(feature = "auth") && (attr.as_deref() == Some("auth") || (attr.is_none() && name == "auth"))
	{
		return Ok(match ty {
			Type::Reference(ty) => MethodArgumentType::AuthStatusRef(*ty.elem),
			ty => MethodArgumentType::AuthStatus(ty)
		});
	}
	
	if cfg!(feature = "database") && (attr.as_deref() == Some("connection") || attr.as_deref() == Some("conn") || (attr.is_none() && name == "conn"))
	{
		return Ok(MethodArgumentType::DatabaseConnection(match ty {
			Type::Reference(ty) => *ty.elem,
			ty => ty
		}));
	}
	
	if index == 0
	{
		return match ty {
			Type::Reference(ty) => Ok(if ty.mutability.is_none() { MethodArgumentType::StateRef } else { MethodArgumentType::StateMutRef }),
			_ => Err(Error::new(ty.span(), "The first argument, unless some feature is used, has to be a (mutable) reference to gotham::state::State"))
		};
	}
	
	Ok(MethodArgumentType::MethodArg(ty))
}

fn interpret_arg(index : usize, arg : &PatType) -> Result<MethodArgument, Error>
{
	let pat = &arg.pat;
	let ident = format_ident!("arg{}", index);
	let orig_name = quote!(#pat);
	let ty = interpret_arg_ty(index, &arg.attrs, &orig_name.to_string(), *arg.ty.clone())?;
	
	Ok(MethodArgument { ident, ident_span: arg.pat.span(), ty })
}

#[cfg(feature = "openapi")]
fn expand_operation_id(attrs : &AttributeArgs) -> TokenStream2
{
	let mut operation_id : Option<&Lit> = None;
	for meta in attrs
	{
		match meta {
			NestedMeta::Meta(Meta::NameValue(kv)) => {
				if kv.path.segments.last().map(|p| p.ident.to_string()) == Some("operation_id".to_owned())
				{
					operation_id = Some(&kv.lit)
				}
			},
			_ => {}
		}
	}
	
	match operation_id {
		Some(operation_id) => quote! {
			fn operation_id() -> Option<String>
			{
				Some(#operation_id.to_string())
			}
		},
		None => quote!()
	}
}

#[cfg(not(feature = "openapi"))]
fn expand_operation_id(_ : &AttributeArgs) -> TokenStream2
{
	quote!()
}

fn expand_wants_auth(attrs : &AttributeArgs, default : bool) -> TokenStream2
{
	let default_lit = Lit::Bool(LitBool { value: default, span: Span::call_site() });
	let mut wants_auth = &default_lit;
	for meta in attrs
	{
		match meta {
			NestedMeta::Meta(Meta::NameValue(kv)) => {
				if kv.path.segments.last().map(|p| p.ident.to_string()) == Some("wants_auth".to_owned())
				{
					wants_auth = &kv.lit
				}
			},
			_ => {}
		}
	}
	
	quote! {
		fn wants_auth() -> bool
		{
			#wants_auth
		}
	}
}

fn expand(method : Method, attrs : TokenStream, item : TokenStream) -> Result<TokenStream2, Error>
{
	let krate = super::krate();
	
	// parse attributes
	let mut method_attrs = parse_macro_input::parse::<AttributeArgs>(attrs)?;
	let resource_path = match method_attrs.remove(0) {
		NestedMeta::Meta(Meta::Path(path)) => path,
		p => return Err(Error::new(p.span(), "Expected name of the Resource struct this method belongs to"))
	};
	let resource_name = resource_path.segments.last().map(|s| s.ident.to_string())
			.ok_or_else(|| Error::new(resource_path.span(), "Resource name must not be empty"))?;
	
	let fun = parse_macro_input::parse::<ItemFn>(item)?;
	let fun_ident = &fun.sig.ident;
	let fun_vis = &fun.vis;
	let fun_is_async = fun.sig.asyncness.is_some();
	
	if let Some(unsafety) = fun.sig.unsafety
	{
		return Err(Error::new(unsafety.span(), "Resource methods must not be unsafe"));
	}
	
	let trait_ident = method.trait_ident();
	let method_ident = method.fn_ident();
	let mod_ident = method.mod_ident(&resource_name);
	let handler_ident = method.handler_struct_ident(&resource_name);
	let setup_ident = method.setup_ident(&resource_name);
	
	let (ret, is_no_content) = match &fun.sig.output {
		ReturnType::Default => (quote!(#krate::NoContent), true),
		ReturnType::Type(_, ty) => (quote!(#ty), false)
	};
	
	// some default idents we'll need
	let state_ident = format_ident!("state");
	let repo_ident = format_ident!("repo");
	let conn_ident = format_ident!("conn");
	let auth_ident = format_ident!("auth");
	let res_ident = format_ident!("res");
	
	// extract arguments into pattern, ident and type
	let args = fun.sig.inputs.iter()
		.enumerate()
		.map(|(i, arg)| match arg {
			FnArg::Typed(arg) => interpret_arg(i, arg),
			FnArg::Receiver(_) => Err(Error::new(arg.span(), "Didn't expect self parameter"))
		})
		.collect_to_result()?;
	
	// extract the generic parameters to use
	let ty_names = method.type_names();
	let ty_len = ty_names.len();
	let generics_args : Vec<&MethodArgument> = args.iter()
		.filter(|arg| (*arg).ty.is_method_arg())
		.collect();
	if generics_args.len() > ty_len
	{
		return Err(Error::new(generics_args[ty_len].span(), "Too many arguments"));
	}
	else if generics_args.len() < ty_len
	{
		return Err(Error::new(fun_ident.span(), "Too few arguments"));
	}
	let generics : Vec<TokenStream2> = generics_args.iter()
		.map(|arg| arg.ty.quote_ty().unwrap())
		.zip(ty_names)
		.map(|(arg, name)| {
			let ident = format_ident!("{}", name);
			quote!(type #ident = #arg;)
		})
		.collect();
	
	// extract the definition of our method
	let mut args_def : Vec<TokenStream2> = args.iter()
		.filter(|arg| (*arg).ty.is_method_arg())
		.map(|arg| {
			let ident = &arg.ident;
			let ty = arg.ty.quote_ty();
			quote!(#ident : #ty)
		}).collect();
	args_def.insert(0, quote!(mut #state_ident : #krate::State));
	
	// extract the arguments to pass over to the supplied method
	let args_pass : Vec<TokenStream2> = args.iter().map(|arg| match (&arg.ty, &arg.ident) {
		(MethodArgumentType::StateRef, _) => quote!(&#state_ident),
		(MethodArgumentType::StateMutRef, _) => quote!(&mut #state_ident),
		(MethodArgumentType::MethodArg(_), ident) => quote!(#ident),
		(MethodArgumentType::DatabaseConnection(_), _) => quote!(&#conn_ident),
		(MethodArgumentType::AuthStatus(_), _) => quote!(#auth_ident),
		(MethodArgumentType::AuthStatusRef(_), _) => quote!(&#auth_ident)
	}).collect();
	
	// prepare the method block
	let mut block = quote!(#fun_ident(#(#args_pass),*));
	let mut state_block = quote!();
	if fun_is_async
	{
		block = quote!(#block.await);
	}
	if is_no_content
	{
		block = quote!(#block; Default::default())
	}
	if let Some(arg) = args.iter().filter(|arg| (*arg).ty.is_database_conn()).nth(0)
	{
		if fun_is_async
		{
			return Err(Error::new(arg.span(), "async fn is not supported when database support is required, consider boxing"));
		}
		let conn_ty = arg.ty.quote_ty();
		state_block = quote! {
			#state_block
			let #repo_ident = <#krate::export::Repo<#conn_ty>>::borrow_from(&#state_ident).clone();
		};
		block = quote! {
			{
				let #res_ident = #repo_ident.run::<_, (#krate::State, #ret), ()>(move |#conn_ident| {
					let #res_ident = { #block };
					Ok((#state_ident, #res_ident))
				}).await.unwrap();
				#state_ident = #res_ident.0;
				#res_ident.1
			}
		};
	}
	if let Some(arg) = args.iter().filter(|arg| (*arg).ty.is_auth_status()).nth(0)
	{
		let auth_ty = arg.ty.quote_ty();
		state_block = quote! {
			#state_block
			let #auth_ident : #auth_ty = <#auth_ty>::borrow_from(&#state_ident).clone();
		};
	}
	
	// prepare the where clause
	let mut where_clause = quote!(#resource_path : #krate::Resource,);
	for arg in args.iter().filter(|arg| (*arg).ty.is_auth_status())
	{
		let auth_ty = arg.ty.quote_ty();
		where_clause = quote!(#where_clause #auth_ty : Clone,);
	}
	
	// attribute generated code
	let operation_id = expand_operation_id(&method_attrs);
	let wants_auth = expand_wants_auth(&method_attrs, args.iter().any(|arg| (*arg).ty.is_auth_status()));
	
	// put everything together
	Ok(quote! {
		#fun
		
		#fun_vis mod #mod_ident
		{
			use super::*;
			
			struct #handler_ident;
			
			impl #krate::ResourceMethod for #handler_ident
			{
				type Res = #ret;
				
				#operation_id
				#wants_auth
			}
			
			impl #krate::#trait_ident for #handler_ident
			where #where_clause
			{
				#(#generics)*
				
				fn #method_ident(#(#args_def),*) -> std::pin::Pin<Box<dyn std::future::Future<Output = (#krate::State, #ret)> + Send>>
				{
					#[allow(unused_imports)]
					use #krate::{export::FutureExt, FromState};
					
					#state_block
					
					async move {
						let #res_ident = { #block };
						(#state_ident, #res_ident)
					}.boxed()
				}
			}
			
			#[deny(dead_code)]
			pub fn #setup_ident<D : #krate::DrawResourceRoutes>(route : &mut D)
			{
				route.#method_ident::<#handler_ident>();
			}
			
		}
	})
}

pub fn expand_method(method : Method, attrs : TokenStream, item : TokenStream) -> TokenStream
{
	expand(method, attrs, item)
		.unwrap_or_else(|err| err.to_compile_error())
		.into()
}
