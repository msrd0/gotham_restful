use crate::{
	resource::*,
	result::*,
	routing::*,
	OpenapiSchema,
	OpenapiType,
	RequestBody
};
use super::{builder::OpenapiBuilder, handler::OpenapiHandler, SECURITY_NAME};
use gotham::{
	pipeline::chain::PipelineHandleChain,
	router::builder::*
};
use indexmap::IndexMap;
use mime::{Mime, STAR_STAR};
use openapiv3::{
	MediaType, Operation, Parameter, ParameterData, ParameterSchemaOrContent, ReferenceOr,
	ReferenceOr::Item, RequestBody as OARequestBody, Response, Responses, Schema, SchemaKind,
	StatusCode, Type
};
use std::panic::RefUnwindSafe;

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
			..Default::default()
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
					name: (*param).to_string(),
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

fn new_operation(
		operation_id : Option<String>,
		default_status : crate::StatusCode,
		accepted_types : Option<Vec<Mime>>,
		schema : ReferenceOr<Schema>,
		params : OperationParams,
		body_schema : Option<ReferenceOr<Schema>>,
		supported_types : Option<Vec<Mime>>,
		requires_auth : bool
) -> Operation
{
	let content = schema_to_content(accepted_types.unwrap_or_else(|| vec![STAR_STAR]), schema);
	
	let mut responses : IndexMap<StatusCode, ReferenceOr<Response>> = IndexMap::new();
	responses.insert(StatusCode::Code(default_status.as_u16()), Item(Response {
		description: default_status.canonical_reason().map(|d| d.to_string()).unwrap_or_default(),
		content,
		..Default::default()
	}));
	
	let request_body = body_schema.map(|schema| Item(OARequestBody {
		description: None,
		content: schema_to_content(supported_types.unwrap_or_else(|| vec![STAR_STAR]), schema),
		required: true
	}));
	
	let mut security = Vec::new();
	if requires_auth
	{
		let mut sec = IndexMap::new();
		sec.insert(SECURITY_NAME.to_owned(), Vec::new());
		security.push(sec);
	}
	
	Operation {
		tags: Vec::new(),
		operation_id,
		parameters: params.into_params(),
		request_body,
		responses: Responses {
			default: None,
			responses
		},
		deprecated: false,
		security,
		..Default::default()
	}
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
				item.get = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::default(), None, None, Handler::wants_auth()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).read_all::<Handler>()
			}
			
			fn read<Handler : ResourceRead>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.get = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::from_path_params(vec!["id"]), None, None, Handler::wants_auth()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).read::<Handler>()
			}
			
			fn search<Handler : ResourceSearch>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();
				
				let path = format!("/{}/search", &self.1);
				let mut item = (self.0).1.remove_path(&self.1);
				item.get = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::from_query_params(Handler::Query::schema()), None, None, Handler::wants_auth()));
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
				item.post = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::default(), Some(body_schema), Handler::Body::supported_types(), Handler::wants_auth()));
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
				item.put = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::default(), Some(body_schema), Handler::Body::supported_types(), Handler::wants_auth()));
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
				item.put = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::from_path_params(vec!["id"]), Some(body_schema), Handler::Body::supported_types(), Handler::wants_auth()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).update::<Handler>()
			}
			
			fn delete_all<Handler : ResourceDeleteAll>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();

				let path = format!("/{}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::default(), None, None, Handler::wants_auth()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).delete_all::<Handler>()
			}
			
			fn delete<Handler : ResourceDelete>(&mut self)
			{
				let schema = (self.0).1.add_schema::<Handler::Res>();

				let path = format!("/{}/{{id}}", &self.1);
				let mut item = (self.0).1.remove_path(&path);
				item.delete = Some(new_operation(Handler::operation_id(), Handler::Res::default_status(), Handler::Res::accepted_types(), schema, OperationParams::from_path_params(vec!["id"]), None, None, Handler::wants_auth()));
				(self.0).1.add_path(path, item);
				
				(&mut *(self.0).0, self.1).delete::<Handler>()
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
		id : isize
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
		let content = schema_to_content(types.unwrap_or_else(|| vec![STAR_STAR]), Item(schema.into_schema()));
		assert!(content.is_empty());
	}
	
	#[test]
	fn raw_schema_to_content()
	{
		let types = Raw::<&str>::accepted_types();
		let schema = <Raw<&str> as OpenapiType>::schema();
		let content = schema_to_content(types.unwrap_or_else(|| vec![STAR_STAR]), Item(schema.into_schema()));
		assert_eq!(content.len(), 1);
		let json = serde_json::to_string(&content.values().nth(0).unwrap()).unwrap();
		assert_eq!(json, r#"{"schema":{"type":"string","format":"binary"}}"#);
	}
}
