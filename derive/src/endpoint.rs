use crate::util::{CollectToResult, ExpectLit, IntoIdent};
use lazy_regex::regex_is_match;
use paste::paste;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::str::FromStr;
use syn::{
	parse::Parse, spanned::Spanned, Attribute, AttributeArgs, Error, Expr, FnArg, ItemFn, LitBool,
	LitStr, Meta, NestedMeta, PatType, Result, ReturnType, Type
};

#[allow(clippy::large_enum_variant)]
pub enum EndpointType {
	ReadAll,
	Read,
	Search,
	Create,
	UpdateAll,
	Update,
	DeleteAll,
	Delete,
	Custom {
		method: Option<Expr>,
		uri: Option<LitStr>,
		params: Option<LitBool>,
		body: Option<LitBool>
	}
}

impl EndpointType {
	pub fn custom() -> Self {
		Self::Custom {
			method: None,
			uri: None,
			params: None,
			body: None
		}
	}
}

macro_rules! endpoint_type_setter {
	($name:ident : $ty:ty) => {
		impl EndpointType {
			paste! {
				fn [<set_ $name>](&mut self, span: Span, [<new_ $name>]: $ty) -> Result<()> {
					match self {
						Self::Custom { $name, .. } if $name.is_some() => {
							Err(Error::new(span, concat!("`", stringify!($name), "` must not appear more than once")))
						},
						Self::Custom { $name, .. } => {
							*$name = Some([<new_ $name>]);
							Ok(())
						},
						_ => Err(Error::new(span, concat!("`", stringify!($name), "` can only be used on custom endpoints")))
					}
				}
			}
		}
	};
}

endpoint_type_setter!(method: Expr);
endpoint_type_setter!(uri: LitStr);
endpoint_type_setter!(params: LitBool);
endpoint_type_setter!(body: LitBool);

impl FromStr for EndpointType {
	type Err = Error;

	fn from_str(str: &str) -> Result<Self> {
		match str {
			"ReadAll" | "read_all" => Ok(Self::ReadAll),
			"Read" | "read" => Ok(Self::Read),
			"Search" | "search" => Ok(Self::Search),
			"Create" | "create" => Ok(Self::Create),
			"ChangeAll" | "change_all" => Ok(Self::UpdateAll),
			"Change" | "change" => Ok(Self::Update),
			"RemoveAll" | "remove_all" => Ok(Self::DeleteAll),
			"Remove" | "remove" => Ok(Self::Delete),
			_ => Err(Error::new(
				Span::call_site(),
				format!("Unknown method: `{str}'")
			))
		}
	}
}

impl EndpointType {
	fn http_method(&self) -> Option<TokenStream> {
		let hyper_method = quote!(::gotham_restful::gotham::hyper::Method);
		match self {
			Self::ReadAll | Self::Read | Self::Search => Some(quote!(#hyper_method::GET)),
			Self::Create => Some(quote!(#hyper_method::POST)),
			Self::UpdateAll | Self::Update => Some(quote!(#hyper_method::PUT)),
			Self::DeleteAll | Self::Delete => Some(quote!(#hyper_method::DELETE)),
			Self::Custom { method, .. } => method.as_ref().map(ToTokens::to_token_stream)
		}
	}

	fn uri(&self) -> Option<TokenStream> {
		match self {
			Self::ReadAll | Self::Create | Self::UpdateAll | Self::DeleteAll => Some(quote!("")),
			Self::Read | Self::Update | Self::Delete => Some(quote!(":id")),
			Self::Search => Some(quote!("search")),
			Self::Custom { uri, .. } => uri.as_ref().map(ToTokens::to_token_stream)
		}
	}

	fn operation_verb(&self) -> TokenStream {
		let some = quote!(::core::option::Option::Some);
		match self {
			Self::ReadAll => quote!(#some("read_all")),
			Self::Read => quote!(#some("read")),
			Self::Search => quote!(#some("search")),
			Self::Create => quote!(#some("create")),
			Self::UpdateAll => quote!(#some("update_all")),
			Self::Update => quote!(#some("update")),
			Self::DeleteAll => quote!(#some("delete_all")),
			Self::Delete => quote!(#some("delete")),
			Self::Custom { .. } => quote!(::core::option::Option::None)
		}
	}

	fn has_placeholders(&self) -> LitBool {
		match self {
			Self::ReadAll | Self::Search | Self::Create | Self::UpdateAll | Self::DeleteAll => {
				LitBool {
					value: false,
					span: Span::call_site()
				}
			},
			Self::Read | Self::Update | Self::Delete => LitBool {
				value: true,
				span: Span::call_site()
			},
			Self::Custom { uri, .. } => LitBool {
				value: uri
					.as_ref()
					.map(|uri| regex_is_match!(r#"(^|/):[^/]+(/|$)"#, &uri.value()))
					.unwrap_or(false),
				span: Span::call_site()
			}
		}
	}

	fn placeholders_ty(&self, arg_ty: Option<&Type>) -> TokenStream {
		match self {
			Self::ReadAll | Self::Search | Self::Create | Self::UpdateAll | Self::DeleteAll => {
				quote!(::gotham_restful::NoopExtractor)
			},
			Self::Read | Self::Update | Self::Delete => {
				quote!(::gotham_restful::private::IdPlaceholder::<#arg_ty>)
			},
			Self::Custom { .. } => {
				if self.has_placeholders().value {
					arg_ty.to_token_stream()
				} else {
					quote!(::gotham_restful::NoopExtractor)
				}
			},
		}
	}

	fn needs_params(&self) -> LitBool {
		match self {
			Self::ReadAll
			| Self::Read
			| Self::Create
			| Self::UpdateAll
			| Self::Update
			| Self::DeleteAll
			| Self::Delete => LitBool {
				value: false,
				span: Span::call_site()
			},
			Self::Search => LitBool {
				value: true,
				span: Span::call_site()
			},
			Self::Custom { params, .. } => params.clone().unwrap_or_else(|| LitBool {
				value: false,
				span: Span::call_site()
			})
		}
	}

	fn params_ty(&self, arg_ty: Option<&Type>) -> TokenStream {
		match self {
			Self::ReadAll
			| Self::Read
			| Self::Create
			| Self::UpdateAll
			| Self::Update
			| Self::DeleteAll
			| Self::Delete => {
				quote!(::gotham_restful::NoopExtractor)
			},
			Self::Search => quote!(#arg_ty),
			Self::Custom { .. } => {
				if self.needs_params().value {
					arg_ty.to_token_stream()
				} else {
					quote!(::gotham_restful::NoopExtractor)
				}
			},
		}
	}

	fn needs_body(&self) -> LitBool {
		match self {
			Self::ReadAll | Self::Read | Self::Search | Self::DeleteAll | Self::Delete => LitBool {
				value: false,
				span: Span::call_site()
			},
			Self::Create | Self::UpdateAll | Self::Update => LitBool {
				value: true,
				span: Span::call_site()
			},
			Self::Custom { body, .. } => body.clone().unwrap_or_else(|| LitBool {
				value: false,
				span: Span::call_site()
			})
		}
	}

	fn body_ty(&self, arg_ty: Option<&Type>) -> TokenStream {
		match self {
			Self::ReadAll | Self::Read | Self::Search | Self::DeleteAll | Self::Delete => {
				quote!(())
			},
			Self::Create | Self::UpdateAll | Self::Update => quote!(#arg_ty),
			Self::Custom { .. } => {
				if self.needs_body().value {
					arg_ty.to_token_stream()
				} else {
					quote!(())
				}
			},
		}
	}
}

#[allow(clippy::large_enum_variant)]
enum HandlerArgType {
	StateRef,
	StateMutRef,
	MethodArg(Type),
	DatabaseConnection(Type),
	AuthStatus(Type),
	AuthStatusRef(Type)
}

impl HandlerArgType {
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
			Self::MethodArg(ty)
			| Self::DatabaseConnection(ty)
			| Self::AuthStatus(ty)
			| Self::AuthStatusRef(ty) => Some(ty),
			_ => None
		}
	}

	fn quote_ty(&self) -> Option<TokenStream> {
		self.ty().map(|ty| quote!(#ty))
	}
}

struct HandlerArg {
	ident_span: Span,
	ty: HandlerArgType
}

impl Spanned for HandlerArg {
	fn span(&self) -> Span {
		self.ident_span
	}
}

fn interpret_arg_ty(attrs: &[Attribute], name: &str, ty: Type) -> Result<HandlerArgType> {
	let attr = attrs
		.iter()
		.find(|arg| {
			arg.path
				.segments
				.iter()
				.any(|path| &path.ident.to_string() == "rest_arg")
		})
		.map(|arg| arg.tokens.to_string());

	// TODO issue a warning for _state usage once diagnostics become stable
	if attr.as_deref() == Some("state") || (attr.is_none() && (name == "state" || name == "_state"))
	{
		return match ty {
			Type::Reference(ty) => Ok(if ty.mutability.is_none() {
				HandlerArgType::StateRef
			} else {
				HandlerArgType::StateMutRef
			}),
			_ => Err(Error::new(
				ty.span(),
				"The state parameter has to be a (mutable) reference to gotham_restful::State"
			))
		};
	}

	if cfg!(feature = "auth")
		&& (attr.as_deref() == Some("auth") || (attr.is_none() && name == "auth"))
	{
		return Ok(match ty {
			Type::Reference(ty) => HandlerArgType::AuthStatusRef(*ty.elem),
			ty => HandlerArgType::AuthStatus(ty)
		});
	}

	if cfg!(feature = "database")
		&& (attr.as_deref() == Some("connection")
			|| attr.as_deref() == Some("conn")
			|| (attr.is_none() && name == "conn"))
	{
		return Ok(HandlerArgType::DatabaseConnection(match ty {
			Type::Reference(ty) => *ty.elem,
			ty => ty
		}));
	}

	Ok(HandlerArgType::MethodArg(ty))
}

fn interpret_arg(_index: usize, arg: &PatType) -> Result<HandlerArg> {
	let pat = &arg.pat;
	let orig_name = quote!(#pat);
	let ty = interpret_arg_ty(&arg.attrs, &orig_name.to_string(), *arg.ty.clone())?;

	Ok(HandlerArg {
		ident_span: arg.pat.span(),
		ty
	})
}

#[cfg(feature = "openapi")]
fn expand_operation_verb(operation_verb: TokenStream) -> Option<TokenStream> {
	Some(quote! {
		fn operation_verb() -> ::core::option::Option<&'static ::core::primitive::str> {
			#operation_verb
		}
	})
}

#[cfg(not(feature = "openapi"))]
fn expand_operation_verb(_: TokenStream) -> Option<TokenStream> {
	None
}

#[cfg(feature = "openapi")]
fn expand_operation_id(operation_id: Option<LitStr>) -> Option<TokenStream> {
	operation_id.map(|operation_id| {
		quote! {
			fn operation_id() -> ::core::option::Option<::std::string::String> {
				::core::option::Option::Some(::std::string::String::from(#operation_id))
			}
		}
	})
}

#[cfg(not(feature = "openapi"))]
fn expand_operation_id(_: Option<LitStr>) -> Option<TokenStream> {
	None
}

fn expand_wants_auth(wants_auth: Option<LitBool>, default: bool) -> TokenStream {
	let wants_auth = wants_auth.unwrap_or_else(|| LitBool {
		value: default,
		span: Span::call_site()
	});

	quote! {
		fn wants_auth() -> ::core::primitive::bool {
			#wants_auth
		}
	}
}

pub fn endpoint_ident(fn_ident: &Ident) -> Ident {
	format_ident!("{}___gotham_restful_endpoint", fn_ident)
}

macro_rules! error_if_not_openapi {
	($($ident:ident),*) => {
		$(
			#[cfg(not(feature = "openapi"))]
			if let Some($ident) = $ident {
				return Err(Error::new(
					$ident.span(),
					concat!("`", stringify!($ident), "` is only supported with the openapi feature")
				));
			}
		)*
	};
}

// clippy doesn't realize that vectors can be used in closures
#[cfg_attr(feature = "cargo-clippy", allow(clippy::needless_collect))]
fn expand_endpoint_type(
	mut ty: EndpointType,
	attrs: AttributeArgs,
	fun: &ItemFn
) -> Result<TokenStream> {
	// reject unsafe functions
	if let Some(unsafety) = fun.sig.unsafety {
		return Err(Error::new(
			unsafety.span(),
			"Endpoint handler methods must not be unsafe"
		));
	}

	// parse arguments
	let mut debug: bool = false;
	let mut operation_id: Option<LitStr> = None;
	let mut schema: Option<Ident> = None;
	let mut status_codes: Option<Ident> = None;
	let mut wants_auth: Option<LitBool> = None;
	for meta in attrs {
		match meta {
			NestedMeta::Meta(Meta::NameValue(kv)) => {
				if kv.path.is_ident("debug") {
					debug = kv.lit.expect_bool()?.value;
				} else if kv.path.is_ident("operation_id") {
					operation_id = Some(kv.lit.expect_str()?);
				} else if kv.path.is_ident("schema") {
					schema = Some(kv.lit.expect_str()?.into_ident())
				} else if kv.path.is_ident("status_codes") {
					status_codes = Some(kv.lit.expect_str()?.into_ident())
				} else if kv.path.is_ident("wants_auth") {
					wants_auth = Some(kv.lit.expect_bool()?);
				} else if kv.path.is_ident("method") {
					ty.set_method(
						kv.path.span(),
						kv.lit.expect_str()?.parse_with(Expr::parse)?
					)?;
				} else if kv.path.is_ident("uri") {
					ty.set_uri(kv.path.span(), kv.lit.expect_str()?)?;
				} else if kv.path.is_ident("params") {
					ty.set_params(kv.path.span(), kv.lit.expect_bool()?)?;
				} else if kv.path.is_ident("body") {
					ty.set_body(kv.path.span(), kv.lit.expect_bool()?)?;
				} else {
					return Err(Error::new(kv.path.span(), "Unknown attribute"));
				}
			},
			_ => return Err(Error::new(meta.span(), "Invalid attribute syntax"))
		}
	}
	error_if_not_openapi!(operation_id, schema, status_codes);
	if schema.is_some() != status_codes.is_some() {
		return Err(Error::new(
			schema
				.map(|s| s.span())
				.unwrap_or_else(|| status_codes.unwrap().span()),
			"`schema` and `status_codes` may only be used together"
		));
	}

	// extract the documentation
	let mut doc: Vec<String> = Vec::new();
	for attr in &fun.attrs {
		if attr.path.is_ident("doc") {
			if let Some(lit) = attr.parse_meta().and_then(|meta| {
				Ok(match meta {
					Meta::NameValue(kv) => Some(kv.lit.expect_str()?),
					_ => None
				})
			})? {
				doc.push(lit.value());
			}
		}
	}
	let doc = doc.join("\n");
	#[allow(unused_variables)]
	let doc = doc.trim();

	#[allow(unused_mut)]
	let mut description = quote!();
	#[cfg(feature = "openapi")]
	if !doc.is_empty() {
		description = quote! {
			fn description() -> ::core::option::Option<::std::string::String> {
				::core::option::Option::Some(::std::string::String::from(#doc))
			}
		};
	}

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

	let fun_vis = &fun.vis;
	let fun_ident = &fun.sig.ident;
	let fun_is_async = fun.sig.asyncness.is_some();

	let ident = endpoint_ident(fun_ident);
	let (output_ty, is_no_content) = match &fun.sig.output {
		ReturnType::Default => (quote!(::gotham_restful::NoContent), true),
		ReturnType::Type(_, ty) => (quote!(#ty), false)
	};
	let output_struct_ident = schema.as_ref().map(|schema_fn| {
		Ident::new(
			&format!("{schema_fn}_gotham_restful_ResponseSchema"),
			Span::call_site()
		)
	});
	let output_struct = schema.map(|schema_fn| {
		let output_struct_ident = output_struct_ident.as_ref().unwrap_or_else(|| unreachable!());
		let status_codes_fn = status_codes.unwrap_or_else(|| unreachable!());

		let schema_call = quote_spanned! { schema_fn.span() =>
			let schema: ::gotham_restful::private::OpenapiSchema = #schema_fn(code);
		};
		let status_codes_call = quote_spanned! { status_codes_fn.span() =>
			let status_codes: ::std::vec::Vec<::gotham_restful::gotham::hyper::StatusCode> = #status_codes_fn();
		};

		quote! {
			#[allow(non_camel_case_types)]
			struct #output_struct_ident(#output_ty);

			impl ::core::convert::From<#output_ty> for #output_struct_ident {
				fn from(o: #output_ty) -> Self {
					Self(o)
				}
			}

			impl ::gotham_restful::IntoResponse for #output_struct_ident
			where
				#output_ty: ::gotham_restful::IntoResponse
			{
				type Err = <#output_ty as ::gotham_restful::IntoResponse>::Err;

				fn into_response(self) -> ::gotham_restful::private::BoxFuture<'static,
						::core::result::Result<::gotham_restful::Response, Self::Err>>
				{
					::gotham_restful::IntoResponse::into_response(self.0)
				}

				fn accepted_types() -> ::core::option::Option<::std::vec::Vec<::gotham_restful::gotham::mime::Mime>> {
					<#output_ty as ::gotham_restful::IntoResponse>::accepted_types()
				}
			}

			impl ::gotham_restful::ResponseSchema for #output_struct_ident {
				fn schema(
					code: ::gotham_restful::gotham::hyper::StatusCode
				) -> ::gotham_restful::private::OpenapiSchema
				{
					#schema_call
					schema
				}

				fn status_codes() -> ::std::vec::Vec<::gotham_restful::gotham::hyper::StatusCode> {
					#status_codes_call
					status_codes
				}
			}
		}
	});
	let (output_typedef, final_return_ty) = match output_struct_ident {
		Some(output_struct_ident) => (
			quote!(type Output = #output_struct_ident;),
			quote!(#output_struct_ident)
		),
		None => (
			quote_spanned!(output_ty.span() => type Output = #output_ty;),
			quote!(#output_ty)
		)
	};

	let arg_tys = args
		.iter()
		.filter(|arg| arg.ty.is_method_arg())
		.collect::<Vec<_>>();
	let mut arg_ty_idx = 0;
	let mut next_arg_ty = |return_none: bool| {
		if return_none {
			return Ok(None);
		}
		if arg_ty_idx >= arg_tys.len() {
			return Err(Error::new(
				fun_ident.span(),
				"Too few arguments for this endpoint handler"
			));
		}
		let ty = arg_tys[arg_ty_idx].ty.ty().unwrap();
		arg_ty_idx += 1;
		Ok(Some(ty))
	};

	let http_method = ty.http_method().ok_or_else(|| {
		Error::new(
			Span::call_site(),
			"Missing `method` attribute (e.g. `#[endpoint(method = \"gotham_restful::gotham::hyper::Method::GET\")]`)"
		)
	})?;
	let uri = ty.uri().ok_or_else(|| {
		Error::new(
			Span::call_site(),
			"Missing `uri` attribute (e.g. `#[endpoint(uri = \"custom_endpoint\")]`)"
		)
	})?;
	let has_placeholders = ty.has_placeholders();
	let placeholder_ty = ty.placeholders_ty(next_arg_ty(!has_placeholders.value)?);
	let placeholder_typedef =
		quote_spanned!(placeholder_ty.span() => type Placeholders = #placeholder_ty;);
	let needs_params = ty.needs_params();
	let params_ty = ty.params_ty(next_arg_ty(!needs_params.value)?);
	let params_typedef = quote_spanned!(params_ty.span() => type Params = #params_ty;);
	let needs_body = ty.needs_body();
	let body_ty = ty.body_ty(next_arg_ty(!needs_body.value)?);
	let body_typedef = quote_spanned!(body_ty.span() => type Body = #body_ty;);

	if arg_ty_idx < arg_tys.len() {
		return Err(Error::new(
			fun_ident.span(),
			"Extra arguments for this endpoint handler"
		));
	}

	let mut handle_args: Vec<TokenStream> = Vec::new();
	if has_placeholders.value {
		if matches!(ty, EndpointType::Custom { .. }) {
			handle_args.push(quote!(placeholders));
		} else {
			handle_args.push(quote!(placeholders.id));
		}
	}
	if needs_params.value {
		handle_args.push(quote!(params));
	}
	if needs_body.value {
		handle_args.push(quote!(body.unwrap()));
	}
	let handle_args = args.iter().map(|arg| match arg.ty {
		HandlerArgType::StateRef | HandlerArgType::StateMutRef => quote!(state),
		HandlerArgType::MethodArg(_) => handle_args.remove(0),
		HandlerArgType::DatabaseConnection(_) => quote!(&conn),
		HandlerArgType::AuthStatus(_) => quote!(auth),
		HandlerArgType::AuthStatusRef(_) => quote!(&auth)
	});

	let expand_handle_content = || {
		let mut state_block = quote!();
		if let Some(arg) = args.iter().find(|arg| arg.ty.is_auth_status()) {
			let auth_ty = arg.ty.quote_ty();
			let auth_borrow = quote_spanned! { auth_ty.span() =>
				<#auth_ty as ::core::clone::Clone>::clone(
					<#auth_ty as ::gotham_restful::gotham::state::FromState>::borrow_from(state)
				)
			};
			state_block = quote! {
				#state_block
				let auth: #auth_ty = #auth_borrow;
			}
		}

		let mut handle_content = quote!(#fun_ident(#(#handle_args),*));
		if fun_is_async {
			if let Some(arg) = args
				.iter()
				.find(|arg| matches!(arg.ty, HandlerArgType::StateRef))
			{
				return Err(Error::new(
					arg.span(),
					"Endpoint handler functions that are async must not take `&State` as an argument, consider taking `&mut State`"
				));
			}
			handle_content = quote!(#handle_content.await);
		}
		if is_no_content {
			handle_content = quote!(#handle_content; <::gotham_restful::NoContent as ::std::default::Default>::default())
		}

		if let Some(arg) = args.iter().find(|arg| arg.ty.is_database_conn()) {
			let conn_ty = arg.ty.quote_ty();
			state_block = quote! {
				#state_block
				let repo = <::gotham_restful::private::Repo<#conn_ty>>::borrow_from(state).clone();
			};
			handle_content = quote! {
				repo.run::<_, _, ()>(move |conn| {
					Ok({ #handle_content })
				}).await.unwrap()
			};
		}

		Ok(quote! {
			use ::gotham_restful::private::FutureExt as _;
			use ::gotham_restful::gotham::state::FromState as _;
			#state_block
			async move {
				#handle_content
			}.map(<#final_return_ty>::from).boxed()
		})
	};
	let handle_content = match expand_handle_content() {
		Ok(content) => content,
		Err(err) => err.to_compile_error()
	};

	let tr8 = if cfg!(feature = "openapi") {
		quote!(::gotham_restful::EndpointWithSchema)
	} else {
		quote!(::gotham_restful::Endpoint)
	};
	let operation_verb = expand_operation_verb(ty.operation_verb());
	let operation_id = expand_operation_id(operation_id);
	let wants_auth = expand_wants_auth(wants_auth, args.iter().any(|arg| arg.ty.is_auth_status()));
	let code = quote! {
		#[doc(hidden)]
		/// `gotham_restful` implementation detail
		#[allow(non_camel_case_types)]
		#fun_vis struct #ident;

		const _: () = {
			#output_struct

			impl #tr8 for #ident {
				fn http_method() -> ::gotham_restful::gotham::hyper::Method {
					#http_method
				}

				fn uri() -> ::std::borrow::Cow<'static, ::core::primitive::str> {
					{ #uri }.into()
				}

				#operation_verb

				#output_typedef

				fn has_placeholders() -> ::core::primitive::bool {
					#has_placeholders
				}
				#placeholder_typedef

				fn needs_params() -> ::core::primitive::bool {
					#needs_params
				}
				#params_typedef

				fn needs_body() -> ::core::primitive::bool {
					#needs_body
				}
				#body_typedef

				fn handle<'a>(
					state: &'a mut ::gotham_restful::gotham::state::State,
					placeholders: Self::Placeholders,
					params: Self::Params,
					body: ::core::option::Option<Self::Body>
				) -> ::gotham_restful::private::BoxFuture<'a, Self::Output> {
					#handle_content
				}

				#operation_id
				#description
				#wants_auth
			}
		};
	};
	if debug {
		eprintln!("{code}");
	}
	Ok(code)
}

pub fn expand_endpoint(ty: EndpointType, attrs: AttributeArgs, fun: ItemFn) -> Result<TokenStream> {
	let endpoint_type = match expand_endpoint_type(ty, attrs, &fun) {
		Ok(code) => code,
		Err(err) => err.to_compile_error()
	};
	Ok(quote! {
		#fun
		#endpoint_type
	})
}
