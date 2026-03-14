use super::{
	builder::OpenapiBuilder,
	handler::{OpenapiDocHandler, OpenapiSpecHandler},
	operation::OperationDescription
};
use crate::{routing::*, EndpointWithSchema, ResourceWithSchema, ResponseSchema};
use gotham::{
	hyper::{Method, StatusCode},
	pipeline::PipelineHandleChain,
	prelude::*,
	router::builder::{RouterBuilder, ScopeBuilder}
};
use lazy_regex::regex_replace_all;
use openapi_type::OpenapiType;
use std::{collections::HashMap, panic::RefUnwindSafe};

/// This trait adds the `openapi_spec` and `openapi_doc` method to an OpenAPI-aware router.
pub trait GetOpenapi {
	/// Register a GET route to `path` that returns the OpenAPI specification in JSON format.
	fn openapi_spec(&mut self, path: &str);

	/// Register a GET route to `path` that returns the OpenAPI documentation in HTML format.
	fn openapi_doc(&mut self, path: &str);
}

#[derive(Debug)]
pub struct OpenapiRouter<'a, D> {
	pub(crate) router: &'a mut D,
	pub(crate) scope: Option<&'a str>,
	pub(crate) openapi_builder: &'a mut OpenapiBuilder
}

macro_rules! implOpenapiRouter {
	($implType:ident) => {
		impl<'a, 'b, C, P> OpenapiRouter<'a, $implType<'b, C, P>>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			pub fn scope<F>(&mut self, path: &str, callback: F)
			where
				F: FnOnce(&mut OpenapiRouter<'_, ScopeBuilder<'_, C, P>>)
			{
				let mut openapi_builder = self.openapi_builder.clone();
				let new_scope = self
					.scope
					.map(|scope| format!("{scope}/{path}").replace("//", "/"));
				self.router.scope(path, |router| {
					let mut router = OpenapiRouter {
						router,
						scope: Some(new_scope.as_ref().map(String::as_ref).unwrap_or(path)),
						openapi_builder: &mut openapi_builder
					};
					callback(&mut router);
				});
			}
		}

		impl<'a, 'b, C, P> GetOpenapi for OpenapiRouter<'a, $implType<'b, C, P>>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn openapi_spec(&mut self, path: &str) {
				self.router
					.get(path)
					.to_new_handler(OpenapiSpecHandler::new(
						self.openapi_builder.openapi.clone()
					));
			}

			fn openapi_doc(&mut self, path: &str) {
				self.router
					.get(path)
					.to_new_handler(OpenapiDocHandler::new(self.openapi_builder.openapi.clone()));
			}
		}

		impl<'a, 'b, C, P> DrawResourcesWithSchema for OpenapiRouter<'a, $implType<'b, C, P>>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R: ResourceWithSchema>(&mut self, mut path: &str) {
				if path.starts_with('/') {
					path = &path[1..];
				}
				R::setup((self, path));
			}
		}

		impl<'a, 'b, C, P> DrawResourceRoutesWithSchema
			for (&mut OpenapiRouter<'a, $implType<'b, C, P>>, &str)
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn endpoint<E: EndpointWithSchema + 'static>(&mut self) {
				let mut responses: HashMap<StatusCode, _> = HashMap::new();
				for code in E::Output::status_codes() {
					responses.insert(
						code,
						E::Output::schema(code)
							.into_iter()
							.map(|mime_schema| {
								(
									mime_schema.mime,
									(self.0).openapi_builder.add_schema(mime_schema.schema)
								)
							})
							.collect()
					);
				}
				let mut path = format!("{}/{}", self.0.scope.unwrap_or_default(), self.1);
				let mut descr = OperationDescription::new::<E>(responses, &path);
				if E::has_placeholders() {
					descr.set_path_params(E::Placeholders::schema());
				}
				if E::needs_params() {
					descr.set_query_params(E::Params::schema());
				}
				if E::needs_body() {
					let body_schema = (self.0).openapi_builder.add_schema(E::Body::schema());
					descr.set_body::<E::Body>(body_schema);
				}

				let uri: &str = &E::uri();
				let uri =
					regex_replace_all!(r#"(^|/):([^/]+)(/|$)"#, uri, |_, prefix, name, suffix| {
						format!("{prefix}{{{name}}}{suffix}")
					});
				if !uri.is_empty() {
					path = format!("{path}/{uri}");
				}

				let op = descr.into_operation();
				let mut item = (self.0).openapi_builder.remove_path(&path);
				match E::http_method() {
					Method::GET => item.get = Some(op),
					Method::PUT => item.put = Some(op),
					Method::POST => item.post = Some(op),
					Method::DELETE => item.delete = Some(op),
					Method::OPTIONS => item.options = Some(op),
					Method::HEAD => item.head = Some(op),
					Method::PATCH => item.patch = Some(op),
					Method::TRACE => item.trace = Some(op),
					method => {
						warn!("Ignoring unsupported method '{method}' in OpenAPI Specification")
					}
				};
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).endpoint::<E>()
			}
		}
	};
}

implOpenapiRouter!(RouterBuilder);
implOpenapiRouter!(ScopeBuilder);
