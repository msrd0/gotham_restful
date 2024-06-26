use super::SECURITY_NAME;
use crate::{response::OrAllTypes, EndpointWithSchema, IntoResponse, RequestBody};
use gotham::{hyper::StatusCode, mime::Mime};
use openapi_type::{
	indexmap::IndexMap,
	openapiv3::{
		MediaType, Operation, Parameter, ParameterData, ParameterSchemaOrContent, ReferenceOr,
		ReferenceOr::Item, RequestBody as OARequestBody, Response, Responses, Schema, SchemaKind,
		StatusCode as OAStatusCode, Type
	},
	OpenapiSchema
};
use std::{borrow::Cow, collections::HashMap};

fn new_parameter_data(
	name: String,
	required: bool,
	schema: ReferenceOr<Box<Schema>>
) -> ParameterData {
	ParameterData {
		name,
		description: None,
		required,
		deprecated: None,
		format: ParameterSchemaOrContent::Schema(schema.unbox()),
		example: None,
		examples: Default::default(),
		explode: None,
		extensions: Default::default()
	}
}

#[derive(Default)]
struct OperationParams {
	path_params: Option<OpenapiSchema>,
	query_params: Option<OpenapiSchema>
}

impl OperationParams {
	// TODO shouldn't this be a custom openapi_type::Visitor
	// rather than this hacky code?
	fn add_path_params(
		path_params: Option<OpenapiSchema>,
		params: &mut Vec<ReferenceOr<Parameter>>
	) {
		let path_params = match path_params {
			Some(pp) => pp.schema,
			None => return
		};
		let path_params = match path_params.schema_kind {
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

	// TODO shouldn't this be a custom openapi_type::Visitor
	// rather than this hacky code?
	fn add_query_params(
		query_params: Option<OpenapiSchema>,
		params: &mut Vec<ReferenceOr<Parameter>>
	) {
		let query_params = match query_params {
			Some(qp) => qp.schema,
			None => return
		};
		let query_params = match query_params.schema_kind {
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

#[derive(Debug, Default)]
pub enum OperationId {
	/// Automatically generate the operation id based on path and operation verb.
	#[default]
	FullAuto,
	/// Automatically generate the operation id based on path and the provided string.
	SemiAuto(Cow<'static, str>),
	/// Use the provided operation id.
	Manual(String)
}

pub(crate) struct OperationDescription {
	operation_id: Option<String>,
	description: Option<String>,

	accepted_types: Option<Vec<Mime>>,
	responses: HashMap<StatusCode, ReferenceOr<Schema>>,
	params: OperationParams,
	body_schema: Option<ReferenceOr<Schema>>,
	supported_types: Option<Vec<Mime>>,
	requires_auth: bool
}

impl OperationDescription {
	/// Create a new operation description for the given endpoint type and schema. If the endpoint
	/// does not specify an operation id, the path is used to generate one.
	pub(crate) fn new<E: EndpointWithSchema>(
		responses: HashMap<StatusCode, ReferenceOr<Schema>>,
		path: &str
	) -> Self {
		let (mut operation_id, op_id_verb) = match E::operation_id() {
			OperationId::FullAuto => (None, E::operation_verb().map(Cow::Borrowed)),
			OperationId::SemiAuto(verb) => (None, Some(verb)),
			OperationId::Manual(id) => (Some(id), None)
		};
		if let Some(verb) = op_id_verb {
			let op_path = path.replace('/', "_");
			let op_path = op_path.trim_start_matches('_');
			if verb.starts_with(op_path) || verb.ends_with(op_path) {
				operation_id = Some(verb.into_owned());
			} else {
				operation_id = Some(format!("{verb}_{op_path}"));
			}
		}

		Self {
			operation_id,
			description: E::description(),

			accepted_types: E::Output::accepted_types(),
			responses,
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

	fn schema_to_content(
		types: Vec<Mime>,
		schema: ReferenceOr<Schema>
	) -> IndexMap<String, MediaType> {
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
			accepted_types,
			responses,
			params,
			body_schema,
			supported_types,
			requires_auth
		) = (
			self.operation_id,
			self.description,
			self.accepted_types,
			self.responses,
			self.params,
			self.body_schema,
			self.supported_types,
			self.requires_auth
		);

		let responses: IndexMap<OAStatusCode, ReferenceOr<Response>> = responses
			.into_iter()
			.map(|(code, schema)| {
				let content =
					Self::schema_to_content(accepted_types.clone().or_all_types(), schema);
				(
					OAStatusCode::Code(code.as_u16()),
					Item(Response {
						description: code
							.canonical_reason()
							.map(|d| d.to_string())
							.unwrap_or_default(),
						content,
						..Default::default()
					})
				)
			})
			.collect();

		let request_body = body_schema.map(|schema| {
			Item(OARequestBody {
				content: Self::schema_to_content(supported_types.or_all_types(), schema),
				required: true,
				..Default::default()
			})
		});

		let mut security = None;
		if requires_auth {
			let mut sec = IndexMap::new();
			sec.insert(SECURITY_NAME.to_owned(), Vec::new());
			security = Some(vec![sec]);
		}

		Operation {
			tags: Vec::new(),
			operation_id,
			description,
			parameters: params.into_params(),
			request_body,
			responses: Responses {
				responses,
				..Default::default()
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
		let schema = <NoContent as ResponseSchema>::schema(StatusCode::NO_CONTENT);
		let content =
			OperationDescription::schema_to_content(types.or_all_types(), Item(schema.schema));
		assert!(content.is_empty());
	}

	#[test]
	fn raw_schema_to_content() {
		let types = Raw::<&str>::accepted_types();
		let schema = <Raw<&str> as ResponseSchema>::schema(StatusCode::OK);
		let content =
			OperationDescription::schema_to_content(types.or_all_types(), Item(schema.schema));
		assert_eq!(content.len(), 1);
		let json = serde_json::to_string(&content.values().nth(0).unwrap()).unwrap();
		assert_eq!(json, r#"{"schema":{"type":"string","format":"binary"}}"#);
	}
}
