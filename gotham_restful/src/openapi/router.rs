use crate::{
	resource::*,
	result::*,
	routing::*,
	OpenapiSchema,
	OpenapiType,
	RequestBody,
	ResourceType
};
use futures::future::ok;
use gotham::{
	extractor::QueryStringExtractor,
	handler::{Handler, HandlerFuture, NewHandler},
	helpers::http::response::create_response,
	pipeline::chain::PipelineHandleChain,
	router::builder::*,
	state::State
};
use hyper::Body;
use indexmap::IndexMap;
use log::error;
use mime::{Mime, APPLICATION_JSON, TEXT_PLAIN};
use openapiv3::{
	Components, MediaType, OpenAPI, Operation, Parameter, ParameterData, ParameterSchemaOrContent, PathItem,
	Paths, ReferenceOr, ReferenceOr::Item, ReferenceOr::Reference, RequestBody as OARequestBody, Response, Responses, Schema,
	SchemaKind, Server, StatusCode, Type
};
use serde::de::DeserializeOwned;
use std::panic::RefUnwindSafe;

/**
This type is required to build routes while adding them to the generated OpenAPI Spec at the
same time. There is no need to use this type directly. See [`WithOpenapi`] on how to do this.

[`WithOpenapi`]: trait.WithOpenapi.html
*/
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
		match self.0.paths.swap_remove(path) {
			Some(Item(item)) => item,
			_ => PathItem::default()
		}
	}

	fn add_path<Path : ToString>(&mut self, path : Path, item : PathItem)
	{
		self.0.paths.insert(path.to_string(), Item(item));
	}

	fn add_schema_impl(&mut self, name : String, mut schema : OpenapiSchema)
	{
		self.add_schema_dependencies(&mut schema.dependencies);
		
		match &mut self.0.components {
			Some(comp) => {
				comp.schemas.insert(name, Item(schema.into_schema()));
			},
			None => {
				let mut comp = Components::default();
				comp.schemas.insert(name, Item(schema.into_schema()));
				self.0.components = Some(comp);
			}
		};
	}

	fn add_schema_dependencies(&mut self, dependencies : &mut IndexMap<String, OpenapiSchema>)
	{
		let keys : Vec<String> = dependencies.keys().map(|k| k.to_string()).collect();
		for dep in keys
		{
			let dep_schema = dependencies.swap_remove(&dep);
			if let Some(dep_schema) = dep_schema
			{
				self.add_schema_impl(dep, dep_schema);
			}
		}
	}
	
	fn add_schema<T : OpenapiType>(&mut self) -> ReferenceOr<Schema>
	{
		let mut schema = T::schema();
		match schema.name.clone() {
			Some(name) => {
				let reference = Reference { reference: format!("#/components/schemas/{}", name) };
				self.add_schema_impl(name, schema);
				reference
			},
			None => {
				self.add_schema_dependencies(&mut schema.dependencies);
				Item(schema.into_schema())
			}
		}
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

/// This trait adds the `get_openapi` method to an OpenAPI-aware router.
pub trait GetOpenapi
{
	fn get_openapi(&mut self, path : &str);
}

fn schema_to_content(types : Vec<Mime>, schema : ReferenceOr<Schema>) -> IndexMap<String, MediaType>
{
	let mut content : IndexMap<String, MediaType> = IndexMap::new();
	for ty in types
	{
		content.insert(ty.to_string(), MediaType {
			schema: Some(schema.clone()),
			example: None,
			examples: IndexMap::new(),
			encoding: IndexMap::new()
		});
	}
	content
}

#[derive(Default)]
struct OperationParams<'a>
{
	path_params : Vec<&'a str>,
	query_params : Option<OpenapiSchema>
}

impl<'a> OperationParams<'a>
{
	fn new(path_params : Vec<&'a str>, query_params : Option<OpenapiSchema>) -> Self
	{
		Self { path_params, query_params }
	}
	
	fn from_path_params(path_params : Vec<&'a str>) -> Self
	{
		Self::new(path_params, None)
	}
	
	fn from_query_params(query_params : OpenapiSchema) -> Self
	{
		Self::new(Vec::new(), Some(query_params))
	}
	
	fn add_path_params(&self, params : &mut Vec<ReferenceOr<Parameter>>)
	{
		for param in &self.path_params
		{
			params.push(Item(Parameter::Path {
				parameter_data: ParameterData {
					name: param.to_string(),
					description: None,
					required: true,
					deprecated: None,
					format: ParameterSchemaOrContent::Schema(Item(String::schema().into_schema())),
					example: None,
					examples: IndexMap::new()
				},
				style: Default::default(),
			}));
		}
	}
	
	fn add_query_params(self, params : &mut Vec<ReferenceOr<Parameter>>)
	{
		let query_params = match self.query_params {
			Some(qp) => qp.schema,
			None => return
		};
		let query_params = match query_params {
			SchemaKind::Type(Type::Object(ty)) => ty,
			_ => panic!("Query Parameters needs to be a plain struct")
		};
		for (name, schema) in query_params.properties
		{
			let required = query_params.required.contains(&name);
			params.push(Item(Parameter::Query {
				parameter_data: ParameterData {
					name,
					description: None,
					required,
					deprecated: None,
					format: ParameterSchemaOrContent::Schema(schema.unbox()),
					example: None,
					examples: IndexMap::new()
				},
				allow_reserved: false,
				style: Default::default(),
				allow_empty_value: None
			}))
		}
	}
	
	fn into_params(self) -> Vec<ReferenceOr<Parameter>>
	{
		let mut params : Vec<ReferenceOr<Parameter>> = Vec::new();
		self.add_path_params(&mut params);
		self.add_query_params(&mut params);
		params
	}
}

fn new_operation(default_status : hyper::StatusCode, accepted_types : Option<Vec<Mime>>, schema : ReferenceOr<Schema>, params : OperationParams, body_schema : Option<ReferenceOr<Schema>>, supported_types : Option<Vec<Mime>>) -> Operation
{
	let content = schema_to_content(accepted_types.unwrap_or_default(), schema);
	
	let mut responses : IndexMap<StatusCode, ReferenceOr<Response>> = IndexMap::new();
	responses.insert(StatusCode::Code(default_status.as_u16()), Item(Response {
		description: default_status.canonical_reason().map(|d| d.to_string()).unwrap_or_default(),
		headers: IndexMap::new(),
		content,
		links: IndexMap::new()
	}));
	
	let request_body = body_schema.map(|schema| Item(OARequestBody {
		description: None,
		content: schema_to_content(supported_types.unwrap_or_default(), schema),
		required: true
	}));
	
	Operation {
		tags: Vec::new(),
		summary: None,
		description: None,
		external_documentation: None,
		operation_id: None, // TODO
		parameters: params.into_params(),
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
				let schema = (self.0).1.add_schema::<Res>();
				
				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::default(), None, None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).read_all::<Handler, Res>()
			}
			
			fn read<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceRead<ID, Res>
			{
				let schema = (self.0).1.add_schema::<Res>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::from_path_params(vec!["id"]), None, None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).read::<Handler, ID, Res>()
			}
			
			fn search<Handler, Query, Res>(&mut self)
			where
				Query : ResourceType + DeserializeOwned + QueryStringExtractor<Body> + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceSearch<Query, Res>
			{
				let schema = (self.0).1.add_schema::<Res>();
				
				let path = format!("/{}/search", &self.1);
				let mut item = (self.0).1.remove_path(&self.1);
				item.get = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::from_query_params(Query::schema()), None, None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).search::<Handler, Query, Res>()
			}
			
			fn create<Handler, Body, Res>(&mut self)
			where
				Body : RequestBody,
				Res : ResourceResult,
				Handler : ResourceCreate<Body, Res>
			{
				let schema = (self.0).1.add_schema::<Res>();
				let body_schema = (self.0).1.add_schema::<Body>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.post = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::default(), Some(body_schema), Body::supported_types()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).create::<Handler, Body, Res>()
			}
			
			fn update_all<Handler, Body, Res>(&mut self)
			where
				Body : RequestBody,
				Res : ResourceResult,
				Handler : ResourceUpdateAll<Body, Res>
			{
				let schema = (self.0).1.add_schema::<Res>();
				let body_schema = (self.0).1.add_schema::<Body>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.put = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::default(), Some(body_schema), Body::supported_types()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).update_all::<Handler, Body, Res>()
			}
			
			fn update<Handler, ID, Body, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Body : RequestBody,
				Res : ResourceResult,
				Handler : ResourceUpdate<ID, Body, Res>
			{
				let schema = (self.0).1.add_schema::<Res>();
				let body_schema = (self.0).1.add_schema::<Body>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.put = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::from_path_params(vec!["id"]), Some(body_schema), Body::supported_types()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).update::<Handler, ID, Body, Res>()
			}
			
			fn delete_all<Handler, Res>(&mut self)
			where
				Res : ResourceResult,
				Handler : ResourceDeleteAll<Res>
			{
				let schema = (self.0).1.add_schema::<Res>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::default(), None, None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).delete_all::<Handler, Res>()
			}
			
			fn delete<Handler, ID, Res>(&mut self)
			where
				ID : DeserializeOwned + Clone + RefUnwindSafe + Send + Sync + 'static,
				Res : ResourceResult,
				Handler : ResourceDelete<ID, Res>
			{
				let schema = (self.0).1.add_schema::<Res>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(new_operation(Res::default_status(), Res::accepted_types(), schema, OperationParams::from_path_params(vec!["id"]), None, None));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1.to_string()).delete::<Handler, ID, Res>()
			}
		}

	}
}

implOpenapiRouter!(RouterBuilder);
implOpenapiRouter!(ScopeBuilder);


#[cfg(test)]
mod test
{
	use crate::ResourceResult;
	use super::*;
	
	#[derive(OpenapiType)]
	#[allow(dead_code)]
	struct QueryParams
	{
		id : i64
	}
	
	#[test]
	fn params_empty()
	{
		let op_params = OperationParams::default();
		let params = op_params.into_params();
		assert!(params.is_empty());
	}
	
	#[test]
	fn params_from_path_params()
	{
		let name = "id";
		let op_params = OperationParams::from_path_params(vec![name]);
		let params = op_params.into_params();
		let json = serde_json::to_string(&params).unwrap();
		assert_eq!(json, format!(r#"[{{"in":"path","name":"{}","required":true,"schema":{{"type":"string"}},"style":"simple"}}]"#, name));
	}
	
	#[test]
	fn params_from_query_params()
	{
		let op_params = OperationParams::from_query_params(QueryParams::schema());
		let params = op_params.into_params();
		let json = serde_json::to_string(&params).unwrap();
		assert_eq!(json, r#"[{"in":"query","name":"id","required":true,"schema":{"type":"integer"},"style":"form"}]"#);
	}
	
	#[test]
	fn params_both()
	{
		let name = "id";
		let op_params = OperationParams::new(vec![name], Some(QueryParams::schema()));
		let params = op_params.into_params();
		let json = serde_json::to_string(&params).unwrap();
		assert_eq!(json, format!(r#"[{{"in":"path","name":"{}","required":true,"schema":{{"type":"string"}},"style":"simple"}},{{"in":"query","name":"id","required":true,"schema":{{"type":"integer"}},"style":"form"}}]"#, name));
	}
	
	#[test]
	fn no_content_schema_to_content()
	{
		let types = NoContent::accepted_types();
		let schema = <NoContent as OpenapiType>::schema();
		let content = schema_to_content(types.unwrap_or_default(), Item(schema.into_schema()));
		assert!(content.is_empty());
	}
	
	#[test]
	fn raw_schema_to_content()
	{
		let types = Raw::<&str>::accepted_types();
		let schema = <Raw<&str> as OpenapiType>::schema();
		let content = schema_to_content(types.unwrap_or_default(), Item(schema.into_schema()));
		assert_eq!(content.len(), 1);
		let json = serde_json::to_string(&content.values().nth(0).unwrap()).unwrap();
		assert_eq!(json, r#"{"schema":{"type":"string","format":"binary"}}"#);
	}
}