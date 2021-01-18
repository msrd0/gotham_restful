use super::{builder::OpenapiBuilder, handler::OpenapiHandler, operation::OperationDescription};
use crate::{routing::*, EndpointWithSchema, OpenapiType, ResourceWithSchema};
use gotham::{hyper::Method, pipeline::chain::PipelineHandleChain, router::builder::*};
use once_cell::sync::Lazy;
use regex::{Captures, Regex};
use std::panic::RefUnwindSafe;

/// This trait adds the `get_openapi` method to an OpenAPI-aware router.
pub trait GetOpenapi {
	fn get_openapi(&mut self, path: &str);
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
				let new_scope = self.scope.map(|scope| format!("{}/{}", scope, path).replace("//", "/"));
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
			fn get_openapi(&mut self, path: &str) {
				self.router
					.get(path)
					.to_new_handler(OpenapiHandler::new(self.openapi_builder.openapi.clone()));
			}
		}

		impl<'a, 'b, C, P> DrawResourcesWithSchema for OpenapiRouter<'a, $implType<'b, C, P>>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R: ResourceWithSchema>(&mut self, path: &str) {
				R::setup((self, path));
			}
		}

		impl<'a, 'b, C, P> DrawResourceRoutesWithSchema for (&mut OpenapiRouter<'a, $implType<'b, C, P>>, &str)
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn endpoint<E: EndpointWithSchema + 'static>(&mut self) {
				let schema = (self.0).openapi_builder.add_schema::<E::Output>();
				let mut descr = OperationDescription::new::<E>(schema);
				if E::has_placeholders() {
					descr.set_path_params(E::Placeholders::schema());
				}
				if E::needs_params() {
					descr.set_query_params(E::Params::schema());
				}
				if E::needs_body() {
					let body_schema = (self.0).openapi_builder.add_schema::<E::Body>();
					descr.set_body::<E::Body>(body_schema);
				}

				static URI_PLACEHOLDER_REGEX: Lazy<Regex> =
					Lazy::new(|| Regex::new(r#"(^|/):(?P<name>[^/]+)(/|$)"#).unwrap());
				let uri: &str = &E::uri();
				let uri =
					URI_PLACEHOLDER_REGEX.replace_all(uri, |captures: &Captures<'_>| format!("{{{}}}", &captures["name"]));
				let path = if uri.is_empty() {
					format!("{}/{}", self.0.scope.unwrap_or_default(), self.1)
				} else {
					format!("{}/{}/{}", self.0.scope.unwrap_or_default(), self.1, uri)
				};

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
					method => warn!("Ignoring unsupported method '{}' in OpenAPI Specification", method)
				};
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).endpoint::<E>()
			}
		}
	};
}

implOpenapiRouter!(RouterBuilder);
implOpenapiRouter!(ScopeBuilder);
