use crate::{
	resource::*,
	result::*,
	OpenapiSchema,
	RequestBody
};
use super::SECURITY_NAME;
use indexmap::IndexMap;
use mime::{Mime, STAR_STAR};
use openapiv3::{
	MediaType, Operation, Parameter, ParameterData, ParameterSchemaOrContent, ReferenceOr,
	ReferenceOr::Item, RequestBody as OARequestBody, Response, Responses, Schema, SchemaKind,
	StatusCode, Type
};


#[derive(Default)]
struct OperationParams<'a>
{
	path_params : Vec<(&'a str, ReferenceOr<Schema>)>,
	query_params : Option<OpenapiSchema>
}

impl<'a> OperationParams<'a>
{
	fn add_path_params(&self, params : &mut Vec<ReferenceOr<Parameter>>)
	{
		for param in &self.path_params
		{
			params.push(Item(Parameter::Path {
				parameter_data: ParameterData {
					name: (*param).0.to_string(),
					description: None,
					required: true,
					deprecated: None,
					format: ParameterSchemaOrContent::Schema((*param).1.clone()),
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

pub struct OperationDescription<'a>
{
	operation_id : Option<String>,
	default_status : crate::StatusCode,
	accepted_types : Option<Vec<Mime>>,
	schema : ReferenceOr<Schema>,
	params : OperationParams<'a>,
	body_schema : Option<ReferenceOr<Schema>>,
	supported_types : Option<Vec<Mime>>,
	requires_auth : bool
}

impl<'a> OperationDescription<'a>
{
	pub fn new<Handler : ResourceMethod>(schema : ReferenceOr<Schema>) -> Self
	{
		Self {
			operation_id: Handler::operation_id(),
			default_status: Handler::Res::default_status(),
			accepted_types: Handler::Res::accepted_types(),
			schema,
			params: Default::default(),
			body_schema: None,
			supported_types: None,
			requires_auth: Handler::wants_auth()
		}
	}
	
	pub fn add_path_param(mut self, name : &'a str, schema : ReferenceOr<Schema>) -> Self
	{
		self.params.path_params.push((name, schema));
		self
	}
	
	pub fn with_query_params(mut self, params : OpenapiSchema) -> Self
	{
		self.params.query_params = Some(params);
		self
	}
	
	pub fn with_body<Body : RequestBody>(mut self, schema : ReferenceOr<Schema>) -> Self
	{
		self.body_schema = Some(schema);
		self.supported_types = Body::supported_types();
		self
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
	
	pub fn into_operation(self) -> Operation
	{
		// this is unfortunately neccessary to prevent rust from complaining about partially moving self
		let (operation_id, default_status, accepted_types, schema, params, body_schema, supported_types, requires_auth) = (
			self.operation_id, self.default_status, self.accepted_types, self.schema, self.params, self.body_schema, self.supported_types, self.requires_auth);
		
		let content = Self::schema_to_content(accepted_types.unwrap_or_else(|| vec![STAR_STAR]), schema);
		
		let mut responses : IndexMap<StatusCode, ReferenceOr<Response>> = IndexMap::new();
		responses.insert(StatusCode::Code(default_status.as_u16()), Item(Response {
			description: default_status.canonical_reason().map(|d| d.to_string()).unwrap_or_default(),
			content,
			..Default::default()
		}));
		
		let request_body = body_schema.map(|schema| Item(OARequestBody {
			description: None,
			content: Self::schema_to_content(supported_types.unwrap_or_else(|| vec![STAR_STAR]), schema),
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
}


#[cfg(test)]
mod test
{
	use crate::{OpenapiType, ResourceResult};
	use super::*;
	
	#[test]
	fn no_content_schema_to_content()
	{
		let types = NoContent::accepted_types();
		let schema = <NoContent as OpenapiType>::schema();
		let content = OperationDescription::schema_to_content(types.unwrap_or_else(|| vec![STAR_STAR]), Item(schema.into_schema()));
		assert!(content.is_empty());
	}
	
	#[test]
	fn raw_schema_to_content()
	{
		let types = Raw::<&str>::accepted_types();
		let schema = <Raw<&str> as OpenapiType>::schema();
		let content = OperationDescription::schema_to_content(types.unwrap_or_else(|| vec![STAR_STAR]), Item(schema.into_schema()));
		assert_eq!(content.len(), 1);
		let json = serde_json::to_string(&content.values().nth(0).unwrap()).unwrap();
		assert_eq!(json, r#"{"schema":{"type":"string","format":"binary"}}"#);
	}
}
