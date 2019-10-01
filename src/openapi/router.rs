use crate::{
	resource::*,
	result::*,
	routing::*,
	OpenapiType,
	ResourceType
};
use futures::future::ok;
use gotham::{
	handler::{Handler, HandlerFuture, NewHandler},
	helpers::http::response::create_response,
	pipeline::chain::PipelineHandleChain,
	router::builder::*,
	state::State
};
use indexmap::IndexMap;
use log::error;
use mime::{APPLICATION_JSON, TEXT_PLAIN};
use openapiv3::{
	Components, MediaType, OpenAPI, Operation, Parameter, ParameterData, ParameterSchemaOrContent, PathItem,
	PathStyle, Paths, ReferenceOr, ReferenceOr::Item, ReferenceOr::Reference, RequestBody, Response, Responses,
	Server, StatusCode
};
use serde::de::DeserializeOwned;
use std::panic::RefUnwindSafe;

pub struct OpenapiRouter(OpenAPI);

impl OpenapiRouter
{
	pub fn new<Title : ToString, Version : ToString, Url : ToString>(title : Title, version : Version, server_url : Url) -> Self
	{
		Self(OpenAPI {
			openapi: "3.0.2".to_string(),
			info: openapiv3::Info {
				title: title.to_string(),
				description: None,
				terms_of_service: None,
				contact: None,
				license: None,
				version: version.to_string()
			},
			servers: vec![Server {
				url: server_url.to_string(),
				description: None,
				variables: None
			}],
			paths: Paths::new(),
			components: None,
			security: Vec::new(),
			tags: Vec::new(),
			external_docs: None
		})
	}

	/// Remove path from the OpenAPI spec, or return an empty one if not included. This is handy if you need to
	/// modify the path and add it back after the modification
	fn remove_path(&mut self, path : &str) -> PathItem
	{
		if let Some(Item(item)) = self.0.paths.swap_remove(path)
		{
			return item;
		}
		return PathItem::default()
	}

	fn add_path<Path : ToString>(&mut self, path : Path, item : PathItem)
	{
		self.0.paths.insert(path.to_string(), Item(item));
	}

	fn add_schema<T : OpenapiType>(&mut self, path : &str, method : &str, desc : &str) -> String
	{
		let schema = T::to_schema();
		let name = schema.name.clone().unwrap_or_else(|| format!("path_{}_{}_{}", path, method, desc));
		let item = schema.to_schema();
		match &mut self.0.components {
			Some(comp) => {
				comp.schemas.insert(name.to_string(), Item(item));
			},
			None => {
				let mut comp = Components::default();
				comp.schemas.insert(name.to_string(), Item(item));
				self.0.components = Some(comp);
			}
		};
		name
	}
}

#[derive(Clone)]
struct OpenapiHandler(Result<String, String>);

// dunno what/why/whatever
impl RefUnwindSafe for OpenapiHandler {}

impl OpenapiHandler
{
	fn new(openapi : &OpenapiRouter) -> Self
	{
		Self(serde_json::to_string(&openapi.0).map_err(|e| format!("{}", e)))
	}
}

impl NewHandler for OpenapiHandler
{
	type Instance = Self;
	
	fn new_handler(&self) -> gotham::error::Result<Self::Instance>
	{
		Ok(self.clone())
	}
}

impl Handler for OpenapiHandler
{
	fn handle(self, state : State) -> Box<HandlerFuture>
	{
		match self.0 {
			Ok(body) => {
				let res = create_response(&state, hyper::StatusCode::OK, APPLICATION_JSON, body);
				Box::new(ok((state, res)))
			},
			Err(e) => {
				error!("Unable to handle OpenAPI request due to error: {}", e);
				let res = create_response(&state, hyper::StatusCode::INTERNAL_SERVER_ERROR, TEXT_PLAIN, "");
				Box::new(ok((state, res)))
			}
		}
	}
}

pub trait GetOpenapi
{
	fn get_openapi(&mut self, path : &str);
}

fn schema_to_content(schema : &str) -> IndexMap<String, MediaType>
{
	let mut content : IndexMap<String, MediaType> = IndexMap::new();
	content.insert(APPLICATION_JSON.to_string(), MediaType {
		schema: Some(Reference {
			reference: format!("#/components/schemas/{}", schema)
		}),
		example: None,
		examples: IndexMap::new(),
		encoding: IndexMap::new()
	});
	content
}

fn new_operation(schema : &str, path_params : Vec<&str>, body_schema : Option<&str>) -> Operation
{
	let mut responses : IndexMap<StatusCode, ReferenceOr<Response>> = IndexMap::new();
	responses.insert(StatusCode::Code(200), Item(Response {
		description: "OK".to_string(),
		headers: IndexMap::new(),
		content: schema_to_content(schema),
		links: IndexMap::new()
	}));
	
	let mut params : Vec<ReferenceOr<Parameter>> = Vec::new();
	for param in path_params
	{
		params.push(Item(Parameter::Path {
			parameter_data: ParameterData {
				name: param.to_string(),
				description: None,
				required: true,
				deprecated: None,
				format: ParameterSchemaOrContent::Schema(Item(String::to_schema().to_schema())),
				example: None,
				examples: IndexMap::new()
			},
			style: PathStyle::default(),
		}));
	}

	let request_body = body_schema.map(|schema| Item(RequestBody {
		description: None,
		content: schema_to_content(schema),
		required: true
	}));
	
	Operation {
		tags: Vec::new(),
		summary: None,
		description: None,
		external_documentation: None,
		operation_id: None, // TODO
		parameters: params,
		request_body,
		responses: Responses {
			default: None,
			responses
		},
		deprecated: false,
		security: Vec::new(),
		servers: Vec::new()
	}
}

macro_rules! implOpenapiRouter {
	($implType:ident) => {

		impl<'a, C, P> GetOpenapi for (&mut $implType<'a, C, P>, &mut OpenapiRouter)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn get_openapi(&mut self, path : &str)
			{
				self.0.get(path).to_new_handler(OpenapiHandler::new(&self.1));
			}
		}
		
		impl<'a, C, P> DrawResources for (&mut $implType<'a, C, P>, &mut OpenapiRouter)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn resource<R : Resource, T : ToString>(&mut self, path : T)
			{
				R::setup((self, path.to_string()));
			}
		}

		impl<'a, C, P> DrawResourceRoutes for (&mut (&mut $implType<'a, C, P>, &mut OpenapiRouter), String)
		where
			C : PipelineHandleChain<P> + Copy + Send + Sync + 'static,
			P : RefUnwindSafe + Send + Sync + 'static
		{
			fn read_all<Handler, Res>(&mut self)
			where
				Res : ResourceResult,
				Handler : ResourceReadAll<Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "read_all", "result_body");
				
				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(new_operation(&schema, vec![], None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).read_all::<Handler, Res>()
			}
			
			fn read<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceRead<ID, Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "read", "result_body");

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(new_operation(&schema, vec!["id"], None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).read::<Handler, ID, Res>()
			}
			
			fn create<Handler, Body, Res>(&mut self)
			where
				Body : ResourceType,
				Res : ResourceResult,
				Handler : ResourceCreate<Body, Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "create", "result_body");
				let body_schema = (self.0).1.add_schema::<Body>(&self.1, "create", "body");

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.post = Some(new_operation(&schema, vec![], Some(&body_schema)));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).create::<Handler, Body, Res>()
			}
			
			fn update_all<Handler, Body, Res>(&mut self)
			where
				Body : ResourceType,
				Res : ResourceResult,
				Handler : ResourceUpdateAll<Body, Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "update_all", "result_body");
				let body_schema = (self.0).1.add_schema::<Body>(&self.1, "create", "body");

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.put = Some(new_operation(&schema, vec![], Some(&body_schema)));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).update_all::<Handler, Body, Res>()
			}
			
			fn update<Handler, ID, Body, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Body : ResourceType,
				Res : ResourceResult,
				Handler : ResourceUpdate<ID, Body, Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "update", "result_body");
				let body_schema = (self.0).1.add_schema::<Body>(&self.1, "create", "body");

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.put = Some(new_operation(&schema, vec!["id"], Some(&body_schema)));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).update::<Handler, ID, Body, Res>()
			}
			
			fn delete_all<Handler, Res>(&mut self)
			where
				Res : ResourceResult,
				Handler : ResourceDeleteAll<Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "delete_all", "result_body");

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(new_operation(&schema, vec![], None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).delete_all::<Handler, Res>()
			}
			
			fn delete<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceDelete<ID, Res>
			{
				let schema = (self.0).1.add_schema::<Res>(&self.1, "delete", "result_body");

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(new_operation(&schema, vec!["id"], None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).delete::<Handler, ID, Res>()
			}
		}

	}
}

implOpenapiRouter!(RouterBuilder);
implOpenapiRouter!(ScopeBuilder);
