use super::SECURITY_NAME;
use crate::{response::OrAllTypes, EndpointWithSchema, IntoResponse, RequestBody, ResponseSchema};
use indexmap::IndexMap;
use mime::Mime;
use openapi_type::OpenapiSchema;
use openapiv3::{
	MediaType, Operation, Parameter, ParameterData, ParameterSchemaOrContent, ReferenceOr, ReferenceOr::Item,
	RequestBody as OARequestBody, Response, Responses, Schema, SchemaKind, StatusCode, Type
};

fn new_parameter_data(name: String, required: bool, schema: ReferenceOr<Box<Schema>>) -> ParameterData {
	ParameterData {
		name,
		description: None,
		required,
		deprecated: None,
		format: ParameterSchemaOrContent::Schema(schema.unbox()),
		example: None,
		examples: Default::default(),
		extensions: Default::default()
	}
}

#[derive(Default)]
struct OperationParams {
	path_params: Option<OpenapiSchema>,
	query_params: Option<OpenapiSchema>
}

impl OperationParams {
	fn add_path_params(path_params: Option<OpenapiSchema>, params: &mut Vec<ReferenceOr<Parameter>>) {
		let path_params = match path_params {
			Some(pp) => pp.schema,
			None => return
		};
		let path_params = match path_params {
			SchemaKind::Type(Type::Object(ty)) => ty,
			_ => panic!("Path Parameters needs to be a plain struct")
		};
		for (name, schema) in path_params.properties {
			let required = path_params.required.contains(&name);
			params.push(Item(Parameter::Path {
				parameter_data: new_parameter_data(name, required, schema),
				style: Default::default()
			}))
		}
	}

	fn add_query_params(query_params: Option<OpenapiSchema>, params: &mut Vec<ReferenceOr<Parameter>>) {
		let query_params = match query_params {
			Some(qp) => qp.schema,
			None => return
		};
		let query_params = match query_params {
			SchemaKind::Type(Type::Object(ty)) => ty,
			_ => panic!("Query Parameters needs to be a plain struct")
		};
		for (name, schema) in query_params.properties {
			let required = query_params.required.contains(&name);
			params.push(Item(Parameter::Query {
				parameter_data: new_parameter_data(name, required, schema),
				allow_reserved: false,
				style: Default::default(),
				allow_empty_value: None
			}))
		}
	}

	fn into_params(self) -> Vec<ReferenceOr<Parameter>> {
		let mut params: Vec<ReferenceOr<Parameter>> = Vec::new();
		Self::add_path_params(self.path_params, &mut params);
		Self::add_query_params(self.query_params, &mut params);
		params
	}
}

pub(crate) struct OperationDescription {
	operation_id: Option<String>,
	description: Option<String>,
	default_status: gotham::hyper::StatusCode,
	accepted_types: Option<Vec<Mime>>,
	schema: ReferenceOr<Schema>,
	params: OperationParams,
	body_schema: Option<ReferenceOr<Schema>>,
	supported_types: Option<Vec<Mime>>,
	requires_auth: bool
}

impl OperationDescription {
	/// Create a new operation description for the given endpoint type and schema. If the endpoint
	/// does not specify an operation id, the path is used to generate one.
	pub(crate) fn new<E: EndpointWithSchema>(schema: ReferenceOr<Schema>, path: &str) -> Self {
		let operation_id = E::operation_id().or_else(|| {
			E::operation_verb().map(|verb| format!("{}_{}", verb, path.replace("/", "_").trim_start_matches('_')))
		});
		Self {
			operation_id,
			description: E::description(),
			default_status: E::Output::default_status(),
			accepted_types: E::Output::accepted_types(),
			schema,
			params: Default::default(),
			body_schema: None,
			supported_types: None,
			requires_auth: E::wants_auth()
		}
	}

	pub(crate) fn set_path_params(&mut self, params: OpenapiSchema) {
		self.params.path_params = Some(params);
	}

	pub(crate) fn set_query_params(&mut self, params: OpenapiSchema) {
		self.params.query_params = Some(params);
	}

	pub(crate) fn set_body<Body: RequestBody>(&mut self, schema: ReferenceOr<Schema>) {
		self.body_schema = Some(schema);
		self.supported_types = Body::supported_types();
	}

	fn schema_to_content(types: Vec<Mime>, schema: ReferenceOr<Schema>) -> IndexMap<String, MediaType> {
		let mut content: IndexMap<String, MediaType> = IndexMap::new();
		for ty in types {
			content.insert(ty.to_string(), MediaType {
				schema: Some(schema.clone()),
				..Default::default()
			});
		}
		content
	}

	pub(crate) fn into_operation(self) -> Operation {
		// this is unfortunately neccessary to prevent rust from complaining about partially moving self
		let (
			operation_id,
			description,
			default_status,
			accepted_types,
			schema,
			params,
			body_schema,
			supported_types,
			requires_auth
		) = (
			self.operation_id,
			self.description,
			self.default_status,
			self.accepted_types,
			self.schema,
			self.params,
			self.body_schema,
			self.supported_types,
			self.requires_auth
		);

		let content = Self::schema_to_content(accepted_types.or_all_types(), schema);

		let mut responses: IndexMap<StatusCode, ReferenceOr<Response>> = IndexMap::new();
		responses.insert(
			StatusCode::Code(default_status.as_u16()),
			Item(Response {
				description: default_status.canonical_reason().map(|d| d.to_string()).unwrap_or_default(),
				content,
				..Default::default()
			})
		);

		let request_body = body_schema.map(|schema| {
			Item(OARequestBody {
				content: Self::schema_to_content(supported_types.or_all_types(), schema),
				required: true,
				..Default::default()
			})
		});

		let mut security = Vec::new();
		if requires_auth {
			let mut sec = IndexMap::new();
			sec.insert(SECURITY_NAME.to_owned(), Vec::new());
			security.push(sec);
		}

		Operation {
			tags: Vec::new(),
			operation_id,
			description,
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
mod test {
	use super::*;
	use crate::{NoContent, Raw, ResponseSchema};

	#[test]
	fn no_content_schema_to_content() {
		let types = NoContent::accepted_types();
		let schema = <NoContent as ResponseSchema>::schema();
		let content = OperationDescription::schema_to_content(types.or_all_types(), Item(schema.into_schema()));
		assert!(content.is_empty());
	}

	#[test]
	fn raw_schema_to_content() {
		let types = Raw::<&str>::accepted_types();
		let schema = <Raw<&str> as ResponseSchema>::schema();
		let content = OperationDescription::schema_to_content(types.or_all_types(), Item(schema.into_schema()));
		assert_eq!(content.len(), 1);
		let json = serde_json::to_string(&content.values().nth(0).unwrap()).unwrap();
		assert_eq!(json, r#"{"schema":{"type":"string","format":"binary"}}"#);
	}
}
