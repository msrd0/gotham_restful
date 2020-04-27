use crate::{
	resource::*,
	routing::*,
	OpenapiType,
};
use super::{builder::OpenapiBuilder, handler::OpenapiHandler, operation::OperationDescription};
use gotham::{
	pipeline::chain::PipelineHandleChain,
	router::builder::*
};
use std::panic::RefUnwindSafe;

/// This trait adds the `get_openapi` method to an OpenAPI-aware router.
pub trait GetOpenapi
{
	fn get_openapi(&mut self, path : &str);
}

macro_rules! implOpenapiRouter {
	($implType:ident) => {

		impl<'a, C, P> GetOpenapi for (&mut $implType<'a, C, P>, &mut OpenapiBuilder)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn get_openapi(&mut self, path : &str)
			{
				self.0.get(path).to_new_handler(OpenapiHandler::new(self.1.openapi.clone()));
			}
		}
		
		impl<'a, C, P> DrawResources for (&mut $implType<'a, C, P>, &mut OpenapiBuilder)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R : Resource>(&mut self, path : &str)
			{
				R::setup((self, path));
			}
		}

		impl<'a, C, P> DrawResourceRoutes for (&mut (&mut $implType<'a, C, P>, &mut OpenapiBuilder), &str)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn read_all<Handler : ResourceReadAll>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();
				
				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(OperationDescription::new::<Handler>(schema).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).read_all::<Handler>()
			}
			
			fn read<Handler : ResourceRead>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(OperationDescription::new::<Handler>(schema).with_path_params(vec!["id"]).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).read::<Handler>()
			}
			
			fn search<Handler : ResourceSearch>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();
				
				let path = format!("/{}/search", &self.1);
				let mut item = (self.0).1.remove_path(&self.1);
				item.get = Some(OperationDescription::new::<Handler>(schema).with_query_params(Handler::Query::schema()).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).search::<Handler>()
			}
			
			fn create<Handler : ResourceCreate>(&mut self)
			where
				Handler::Res : 'static,
				Handler::Body : 'static
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();
				let body_schema = (self.0).1.add_schema::<Handler::Body>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.post = Some(OperationDescription::new::<Handler>(schema).with_body::<Handler::Body>(body_schema).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).create::<Handler>()
			}
			
			fn update_all<Handler : ResourceUpdateAll>(&mut self)
			where
				Handler::Res : 'static,
				Handler::Body : 'static
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();
				let body_schema = (self.0).1.add_schema::<Handler::Body>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.put = Some(OperationDescription::new::<Handler>(schema).with_body::<Handler::Body>(body_schema).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).update_all::<Handler>()
			}
			
			fn update<Handler : ResourceUpdate>(&mut self)
			where
				Handler::Res : 'static,
				Handler::Body : 'static
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();
				let body_schema = (self.0).1.add_schema::<Handler::Body>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.put = Some(OperationDescription::new::<Handler>(schema).with_path_params(vec!["id"]).with_body::<Handler::Body>(body_schema).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).update::<Handler>()
			}
			
			fn delete_all<Handler : ResourceDeleteAll>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(OperationDescription::new::<Handler>(schema).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).delete_all::<Handler>()
			}
			
			fn delete<Handler : ResourceDelete>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(OperationDescription::new::<Handler>(schema).with_path_params(vec!["id"]).into_operation());
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).delete::<Handler>()
			}
		}

	}
}

implOpenapiRouter!(RouterBuilder);
implOpenapiRouter!(ScopeBuilder);
