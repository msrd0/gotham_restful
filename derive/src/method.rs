use crate::util::CollectToResult;
use heck::{CamelCase, SnakeCase};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::str::FromStr;
use syn::{
	spanned::Spanned, Attribute, AttributeArgs, Error, FnArg, ItemFn, Lit, LitBool, Meta, NestedMeta, PatType, Result,
	ReturnType, Type
};

pub enum Method {
	ReadAll,
	Read,
	Search,
	Create,
	ChangeAll,
	Change,
	RemoveAll,
	Remove
}

impl FromStr for Method {
	type Err = Error;

	fn from_str(str: &str) -> Result<Self> {
		match str {
			"ReadAll" | "read_all" => Ok(Self::ReadAll),
			"Read" | "read" => Ok(Self::Read),
			"Search" | "search" => Ok(Self::Search),
			"Create" | "create" => Ok(Self::Create),
			"ChangeAll" | "change_all" => Ok(Self::ChangeAll),
			"Change" | "change" => Ok(Self::Change),
			"RemoveAll" | "remove_all" => Ok(Self::RemoveAll),
			"Remove" | "remove" => Ok(Self::Remove),
			_ => Err(Error::new(Span::call_site(), format!("Unknown method: `{}'", str)))
		}
	}
}

impl Method {
	pub fn type_names(&self) -> Vec<&'static str> {
		use Method::*;

		match self {
			ReadAll | RemoveAll => vec![],
			Read | Remove => vec!["ID"],
			Search => vec!["Query"],
			Create | ChangeAll => vec!["Body"],
			Change => vec!["ID", "Body"]
		}
	}

	pub fn trait_ident(&self) -> Ident {
		use Method::*;

		let name = match self {
			ReadAll => "ReadAll",
			Read => "Read",
			Search => "Search",
			Create => "Create",
			ChangeAll => "ChangeAll",
			Change => "Change",
			RemoveAll => "RemoveAll",
			Remove => "Remove"
		};
		format_ident!("Resource{}", name)
	}

	pub fn fn_ident(&self) -> Ident {
		use Method::*;

		let name = match self {
			ReadAll => "read_all",
			Read => "read",
			Search => "search",
			Create => "create",
			ChangeAll => "change_all",
			Change => "change",
			RemoveAll => "remove_all",
			Remove => "remove"
		};
		format_ident!("{}", name)
	}

	pub fn mod_ident(&self, resource: &str) -> Ident {
		format_ident!(
			"_gotham_restful_resource_{}_method_{}",
			resource.to_snake_case(),
			self.fn_ident()
		)
	}

	pub fn handler_struct_ident(&self, resource: &str) -> Ident {
		format_ident!("{}{}Handler", resource.to_camel_case(), self.trait_ident())
	}

	pub fn setup_ident(&self, resource: &str) -> Ident {
		format_ident!("{}_{}_setup_impl", resource.to_snake_case(), self.fn_ident())
	}
}

#[allow(clippy::large_enum_variant)]
enum MethodArgumentType {
	StateRef,
	StateMutRef,
	MethodArg(Type),
	DatabaseConnection(Type),
	AuthStatus(Type),
	AuthStatusRef(Type)
}

impl MethodArgumentType {
	fn is_method_arg(&self) -> bool {
		matches!(self, Self::MethodArg(_))
	}

	fn is_database_conn(&self) -> bool {
		matches!(self, Self::DatabaseConnection(_))
	}

	fn is_auth_status(&self) -> bool {
		matches!(self, Self::AuthStatus(_) | Self::AuthStatusRef(_))
	}

	fn ty(&self) -> Option<&Type> {
		match self {
			Self::MethodArg(ty) | Self::DatabaseConnection(ty) | Self::AuthStatus(ty) | Self::AuthStatusRef(ty) => Some(ty),
			_ => None
		}
	}

	fn quote_ty(&self) -> Option<TokenStream> {
		self.ty().map(|ty| quote!(#ty))
	}
}

struct MethodArgument {
	ident: Ident,
	ident_span: Span,
	ty: MethodArgumentType
}

impl Spanned for MethodArgument {
	fn span(&self) -> Span {
		self.ident_span
	}
}

fn interpret_arg_ty(attrs: &[Attribute], name: &str, ty: Type) -> Result<MethodArgumentType> {
	let attr = attrs
		.iter()
		.find(|arg| arg.path.segments.iter().any(|path| &path.ident.to_string() == "rest_arg"))
		.map(|arg| arg.tokens.to_string());

	// TODO issue a warning for _state usage once diagnostics become stable
	if attr.as_deref() == Some("state") || (attr.is_none() && (name == "state" || name == "_state")) {
		return match ty {
			Type::Reference(ty) => Ok(if ty.mutability.is_none() {
				MethodArgumentType::StateRef
			} else {
				MethodArgumentType::StateMutRef
			}),
			_ => Err(Error::new(
				ty.span(),
				"The state parameter has to be a (mutable) reference to gotham_restful::State"
			))
		};
	}

	if cfg!(feature = "auth") && (attr.as_deref() == Some("auth") || (attr.is_none() && name == "auth")) {
		return Ok(match ty {
			Type::Reference(ty) => MethodArgumentType::AuthStatusRef(*ty.elem),
			ty => MethodArgumentType::AuthStatus(ty)
		});
	}

	if cfg!(feature = "database")
		&& (attr.as_deref() == Some("connection") || attr.as_deref() == Some("conn") || (attr.is_none() && name == "conn"))
	{
		return Ok(MethodArgumentType::DatabaseConnection(match ty {
			Type::Reference(ty) => *ty.elem,
			ty => ty
		}));
	}

	Ok(MethodArgumentType::MethodArg(ty))
}

fn interpret_arg(index: usize, arg: &PatType) -> Result<MethodArgument> {
	let pat = &arg.pat;
	let ident = format_ident!("arg{}", index);
	let orig_name = quote!(#pat);
	let ty = interpret_arg_ty(&arg.attrs, &orig_name.to_string(), *arg.ty.clone())?;

	Ok(MethodArgument {
		ident,
		ident_span: arg.pat.span(),
		ty
	})
}

#[cfg(feature = "openapi")]
fn expand_operation_id(attrs: &[NestedMeta]) -> TokenStream {
	let mut operation_id: Option<&Lit> = None;
	for meta in attrs {
		if let NestedMeta::Meta(Meta::NameValue(kv)) = meta {
			if kv.path.segments.last().map(|p| p.ident.to_string()) == Some("operation_id".to_owned()) {
				operation_id = Some(&kv.lit)
			}
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
fn expand_operation_id(_: &[NestedMeta]) -> TokenStream {
	quote!()
}

fn expand_wants_auth(attrs: &[NestedMeta], default: bool) -> TokenStream {
	let default_lit = Lit::Bool(LitBool {
		value: default,
		span: Span::call_site()
	});
	let mut wants_auth = &default_lit;
	for meta in attrs {
		if let NestedMeta::Meta(Meta::NameValue(kv)) = meta {
			if kv.path.segments.last().map(|p| p.ident.to_string()) == Some("wants_auth".to_owned()) {
				wants_auth = &kv.lit
			}
		}
	}

	quote! {
		fn wants_auth() -> bool
		{
			#wants_auth
		}
	}
}

#[allow(clippy::comparison_chain)]
pub fn expand_method(method: Method, mut attrs: AttributeArgs, fun: ItemFn) -> Result<TokenStream> {
	let krate = super::krate();

	// parse attributes
	if attrs.len() < 1 {
		return Err(Error::new(
			Span::call_site(),
			"Missing Resource struct. Example: #[read_all(MyResource)]"
		));
	}
	let resource_path = match attrs.remove(0) {
		NestedMeta::Meta(Meta::Path(path)) => path,
		p => {
			return Err(Error::new(
				p.span(),
				"Expected name of the Resource struct this method belongs to"
			))
		},
	};
	let resource_name = resource_path
		.segments
		.last()
		.map(|s| s.ident.to_string())
		.ok_or_else(|| Error::new(resource_path.span(), "Resource name must not be empty"))?;

	let fun_ident = &fun.sig.ident;
	let fun_vis = &fun.vis;
	let fun_is_async = fun.sig.asyncness.is_some();

	if let Some(unsafety) = fun.sig.unsafety {
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
	let args = fun
		.sig
		.inputs
		.iter()
		.enumerate()
		.map(|(i, arg)| match arg {
			FnArg::Typed(arg) => interpret_arg(i, arg),
			FnArg::Receiver(_) => Err(Error::new(arg.span(), "Didn't expect self parameter"))
		})
		.collect_to_result()?;

	// extract the generic parameters to use
	let ty_names = method.type_names();
	let ty_len = ty_names.len();
	let generics_args: Vec<&MethodArgument> = args.iter().filter(|arg| (*arg).ty.is_method_arg()).collect();
	if generics_args.len() > ty_len {
		return Err(Error::new(generics_args[ty_len].span(), "Too many arguments"));
	} else if generics_args.len() < ty_len {
		return Err(Error::new(fun_ident.span(), "Too few arguments"));
	}
	let generics: Vec<TokenStream> = generics_args
		.iter()
		.map(|arg| arg.ty.quote_ty().unwrap())
		.zip(ty_names)
		.map(|(arg, name)| {
			let ident = format_ident!("{}", name);
			quote!(type #ident = #arg;)
		})
		.collect();

	// extract the definition of our method
	let mut args_def: Vec<TokenStream> = args
		.iter()
		.filter(|arg| (*arg).ty.is_method_arg())
		.map(|arg| {
			let ident = &arg.ident;
			let ty = arg.ty.quote_ty();
			quote!(#ident : #ty)
		})
		.collect();
	args_def.insert(0, quote!(mut #state_ident : #krate::State));

	// extract the arguments to pass over to the supplied method
	let args_pass: Vec<TokenStream> = args
		.iter()
		.map(|arg| match (&arg.ty, &arg.ident) {
			(MethodArgumentType::StateRef, _) => quote!(&#state_ident),
			(MethodArgumentType::StateMutRef, _) => quote!(&mut #state_ident),
			(MethodArgumentType::MethodArg(_), ident) => quote!(#ident),
			(MethodArgumentType::DatabaseConnection(_), _) => quote!(&#conn_ident),
			(MethodArgumentType::AuthStatus(_), _) => quote!(#auth_ident),
			(MethodArgumentType::AuthStatusRef(_), _) => quote!(&#auth_ident)
		})
		.collect();

	// prepare the method block
	let mut block = quote!(#fun_ident(#(#args_pass),*));
	let mut state_block = quote!();
	if fun_is_async {
		if let Some(arg) = args.iter().find(|arg| matches!((*arg).ty, MethodArgumentType::StateRef)) {
			return Err(Error::new(
				arg.span(),
				"async fn must not take &State as an argument as State is not Sync, consider taking &mut State"
			));
		}
		block = quote!(#block.await);
	}
	if is_no_content {
		block = quote!(#block; Default::default())
	}
	if let Some(arg) = args.iter().find(|arg| (*arg).ty.is_database_conn()) {
		if fun_is_async {
			return Err(Error::new(
				arg.span(),
				"async fn is not supported when database support is required, consider boxing"
			));
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
	if let Some(arg) = args.iter().find(|arg| (*arg).ty.is_auth_status()) {
		let auth_ty = arg.ty.quote_ty();
		state_block = quote! {
			#state_block
			let #auth_ident : #auth_ty = <#auth_ty>::borrow_from(&#state_ident).clone();
		};
	}

	// prepare the where clause
	let mut where_clause = quote!(#resource_path : #krate::Resource,);
	for arg in args.iter().filter(|arg| (*arg).ty.is_auth_status()) {
		let auth_ty = arg.ty.quote_ty();
		where_clause = quote!(#where_clause #auth_ty : Clone,);
	}

	// attribute generated code
	let operation_id = expand_operation_id(&attrs);
	let wants_auth = expand_wants_auth(&attrs, args.iter().any(|arg| (*arg).ty.is_auth_status()));

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
