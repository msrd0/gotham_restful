use indexmap::IndexMap;
use openapi_type::OpenapiSchema;
use openapiv3::{
	Components, OpenAPI, PathItem, ReferenceOr,
	ReferenceOr::{Item, Reference},
	Schema, Server
};
use std::sync::{Arc, RwLock};

#[derive(Clone, Debug)]
pub struct OpenapiInfo {
	pub title: String,
	pub version: String,
	pub urls: Vec<String>
}

#[derive(Clone, Debug)]
pub struct OpenapiBuilder {
	pub openapi: Arc<RwLock<OpenAPI>>
}

impl OpenapiBuilder {
	pub fn new(info: OpenapiInfo) -> Self {
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
	pub fn remove_path(&mut self, path: &str) -> PathItem {
		let mut openapi = self.openapi.write().unwrap();
		match openapi.paths.swap_remove(path) {
			Some(Item(item)) => item,
			_ => PathItem::default()
		}
	}

	pub fn add_path<Path: ToString>(&mut self, path: Path, item: PathItem) {
		let mut openapi = self.openapi.write().unwrap();
		openapi.paths.insert(path.to_string(), Item(item));
	}

	fn add_schema_impl(&mut self, name: String, mut schema: OpenapiSchema) {
		self.add_schema_dependencies(&mut schema.dependencies);

		let mut openapi = self.openapi.write().unwrap();
		match &mut openapi.components {
			Some(comp) => {
				comp.schemas.insert(name, Item(schema.into_schema()));
			},
			None => {
				let mut comp = Components::default();
				comp.schemas.insert(name, Item(schema.into_schema()));
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

	pub fn add_schema(&mut self, mut schema: OpenapiSchema) -> ReferenceOr<Schema> {
		match schema.name.clone() {
			Some(name) => {
				let reference = Reference {
					reference: format!("#/components/schemas/{}", name)
				};
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
			urls: vec!["http://localhost:1234".to_owned(), "https://example.org".to_owned()]
		}
	}

	fn openapi(builder: OpenapiBuilder) -> OpenAPI {
		Arc::try_unwrap(builder.openapi).unwrap().into_inner().unwrap()
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
			ReferenceOr::Item(Message::schema().into_schema())
		);
		assert_eq!(
			openapi.components.clone().unwrap_or_default().schemas["Messages"],
			ReferenceOr::Item(Messages::schema().into_schema())
		);
	}
}
