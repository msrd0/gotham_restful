use openapi_type::{
	indexmap::IndexMap,
	openapiv3::{
		self, Components, OpenAPI, PathItem, ReferenceOr,
		ReferenceOr::{Item, Reference},
		Schema, Server
	},
	OpenapiSchema
};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OpenapiInfo {
	pub title: String,
	pub version: String,
	pub urls: Vec<String>
}

#[derive(Clone, Debug)]
pub(crate) struct OpenapiBuilder {
	pub(crate) openapi: Arc<RwLock<OpenAPI>>
}

impl OpenapiBuilder {
	pub(crate) fn new(info: OpenapiInfo) -> Self {
		Self {
			openapi: Arc::new(RwLock::new(OpenAPI {
				openapi: "3.0.2".to_string(),
				info: openapiv3::Info {
					title: info.title,
					version: info.version,
					..Default::default()
				},
				servers: info
					.urls
					.into_iter()
					.map(|url| Server {
						url,
						..Default::default()
					})
					.collect(),
				..Default::default()
			}))
		}
	}

	/// Remove path from the OpenAPI spec, or return an empty one if not included. This is handy if you need to
	/// modify the path and add it back after the modification
	pub(crate) fn remove_path(&mut self, path: &str) -> PathItem {
		let mut openapi = self.openapi.write();
		match openapi.paths.paths.swap_remove(path) {
			Some(Item(item)) => item,
			_ => PathItem::default()
		}
	}

	pub(crate) fn add_path<Path: ToString>(&mut self, path: Path, item: PathItem) {
		let mut openapi = self.openapi.write();
		openapi.paths.paths.insert(path.to_string(), Item(item));
	}

	fn add_schema_impl(&mut self, name: String, mut schema: OpenapiSchema) {
		self.add_schema_dependencies(&mut schema.dependencies);

		let mut openapi = self.openapi.write();
		match &mut openapi.components {
			Some(comp) => {
				comp.schemas.insert(name, Item(schema.schema));
			},
			None => {
				let mut comp = Components::default();
				comp.schemas.insert(name, Item(schema.schema));
				openapi.components = Some(comp);
			}
		};
	}

	fn add_schema_dependencies(&mut self, dependencies: &mut IndexMap<String, OpenapiSchema>) {
		let keys: Vec<String> = dependencies.keys().map(|k| k.to_string()).collect();
		for dep in keys {
			let dep_schema = dependencies.swap_remove(&dep);
			if let Some(dep_schema) = dep_schema {
				self.add_schema_impl(dep, dep_schema);
			}
		}
	}

	pub(crate) fn add_schema(&mut self, mut schema: OpenapiSchema) -> ReferenceOr<Schema> {
		match schema.schema.schema_data.title.clone() {
			Some(name) => {
				let reference = Reference {
					reference: format!("#/components/schemas/{name}")
				};
				self.add_schema_impl(name, schema);
				reference
			},
			None => {
				self.add_schema_dependencies(&mut schema.dependencies);
				Item(schema.schema)
			}
		}
	}
}

#[cfg(test)]
#[allow(dead_code)]
mod test {
	use super::*;
	use openapi_type::OpenapiType;

	#[derive(OpenapiType)]
	struct Message {
		msg: String
	}

	#[derive(OpenapiType)]
	struct Messages {
		msgs: Vec<Message>
	}

	fn info() -> OpenapiInfo {
		OpenapiInfo {
			title: "TEST CASE".to_owned(),
			version: "1.2.3".to_owned(),
			urls: vec![
				"http://localhost:1234".to_owned(),
				"https://example.org".to_owned(),
			]
		}
	}

	fn openapi(builder: OpenapiBuilder) -> OpenAPI {
		Arc::try_unwrap(builder.openapi).unwrap().into_inner()
	}

	#[test]
	fn new_builder() {
		let info = info();
		let builder = OpenapiBuilder::new(info.clone());
		let openapi = openapi(builder);

		assert_eq!(info.title, openapi.info.title);
		assert_eq!(info.version, openapi.info.version);
		assert_eq!(info.urls.len(), openapi.servers.len());
	}

	#[test]
	fn add_schema() {
		let mut builder = OpenapiBuilder::new(info());
		builder.add_schema(<Option<Messages>>::schema());
		let openapi = openapi(builder);

		assert_eq!(
			openapi.components.clone().unwrap_or_default().schemas["Message"],
			ReferenceOr::Item(Message::schema().schema)
		);
		assert_eq!(
			openapi.components.clone().unwrap_or_default().schemas["Messages"],
			ReferenceOr::Item(Messages::schema().schema)
		);
	}
}
