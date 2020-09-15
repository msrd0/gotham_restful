use super::{builder::OpenapiBuilder, handler::OpenapiHandler, operation::OperationDescription};
use crate::{resource::*, routing::*, OpenapiType};
use gotham::{pipeline::chain::PipelineHandleChain, router::builder::*};
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

		impl<'a, 'b, C, P> DrawResources for OpenapiRouter<'a, $implType<'b, C, P>>
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R: Resource>(&mut self, path: &str) {
				R::setup((self, path));
			}
		}

		impl<'a, 'b, C, P> DrawResourceRoutes for (&mut OpenapiRouter<'a, $implType<'b, C, P>>, &str)
		where
			C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P: RefUnwindSafe + Send + Sync + 'static
		{
			fn read_all<Handler: ResourceReadAll>(&mut self) {
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();

				let path = format!("{}/{}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.get = Some(OperationDescription::new::<Handler>(schema).into_operation());
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).read_all::<Handler>()
			}

			fn read<Handler: ResourceRead>(&mut self) {
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();
				let id_schema = (self.0).openapi_builder.add_schema::<Handler::ID>();

				let path = format!("{}/{}/{{id}}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.get = Some(
					OperationDescription::new::<Handler>(schema)
						.add_path_param("id", id_schema)
						.into_operation()
				);
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).read::<Handler>()
			}

			fn search<Handler: ResourceSearch>(&mut self) {
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();

				let path = format!("{}/{}/search", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.get = Some(
					OperationDescription::new::<Handler>(schema)
						.with_query_params(Handler::Query::schema())
						.into_operation()
				);
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).search::<Handler>()
			}

			fn create<Handler: ResourceCreate>(&mut self)
			where
				Handler::Res: 'static,
				Handler::Body: 'static
			{
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();
				let body_schema = (self.0).openapi_builder.add_schema::<Handler::Body>();

				let path = format!("{}/{}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.post = Some(
					OperationDescription::new::<Handler>(schema)
						.with_body::<Handler::Body>(body_schema)
						.into_operation()
				);
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).create::<Handler>()
			}

			fn change_all<Handler: ResourceChangeAll>(&mut self)
			where
				Handler::Res: 'static,
				Handler::Body: 'static
			{
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();
				let body_schema = (self.0).openapi_builder.add_schema::<Handler::Body>();

				let path = format!("{}/{}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.put = Some(
					OperationDescription::new::<Handler>(schema)
						.with_body::<Handler::Body>(body_schema)
						.into_operation()
				);
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).change_all::<Handler>()
			}

			fn change<Handler: ResourceChange>(&mut self)
			where
				Handler::Res: 'static,
				Handler::Body: 'static
			{
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();
				let id_schema = (self.0).openapi_builder.add_schema::<Handler::ID>();
				let body_schema = (self.0).openapi_builder.add_schema::<Handler::Body>();

				let path = format!("{}/{}/{{id}}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.put = Some(
					OperationDescription::new::<Handler>(schema)
						.add_path_param("id", id_schema)
						.with_body::<Handler::Body>(body_schema)
						.into_operation()
				);
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).change::<Handler>()
			}

			fn remove_all<Handler: ResourceRemoveAll>(&mut self) {
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();

				let path = format!("{}/{}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.delete = Some(OperationDescription::new::<Handler>(schema).into_operation());
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).remove_all::<Handler>()
			}

			fn remove<Handler: ResourceRemove>(&mut self) {
				let schema = (self.0).openapi_builder.add_schema::<Handler::Res>();
				let id_schema = (self.0).openapi_builder.add_schema::<Handler::ID>();

				let path = format!("{}/{}/{{id}}", self.0.scope.unwrap_or_default(), self.1);
				let mut item = (self.0).openapi_builder.remove_path(&path);
				item.delete = Some(
					OperationDescription::new::<Handler>(schema)
						.add_path_param("id", id_schema)
						.into_operation()
				);
				(self.0).openapi_builder.add_path(path, item);

				(&mut *(self.0).router, self.1).remove::<Handler>()
			}
		}
	};
}

implOpenapiRouter!(RouterBuilder);
implOpenapiRouter!(ScopeBuilder);
