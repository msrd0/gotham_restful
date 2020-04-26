use crate::{OpenapiType, OpenapiSchema};
use indexmap::IndexMap;
use openapiv3::{
	Components, OpenAPI, PathItem, ReferenceOr, ReferenceOr::Item, ReferenceOr::Reference, Schema,
	Server
};
use std::sync::{Arc, RwLock};

pub struct OpenapiBuilder
{
	pub openapi : Arc<RwLock<OpenAPI>>
}

impl OpenapiBuilder
{
	pub fn new(title : String, version : String, url : String) -> Self
	{
		Self {
			openapi: Arc::new(RwLock::new(OpenAPI {
				openapi: "3.0.2".to_string(),
				info: openapiv3::Info {
					title, version,
					..Default::default()
				},
				servers: vec![Server {
					url,
					..Default::default()
				}],
				..Default::default()
			}))
		}
	}
	
	/// Remove path from the OpenAPI spec, or return an empty one if not included. This is handy if you need to
	/// modify the path and add it back after the modification
	pub fn remove_path(&mut self, path : &str) -> PathItem
	{
		let mut openapi = self.openapi.write().unwrap();
		match openapi.paths.swap_remove(path) {
			Some(Item(item)) => item,
			_ => PathItem::default()
		}
	}

	pub fn add_path<Path : ToString>(&mut self, path : Path, item : PathItem)
	{
		let mut openapi = self.openapi.write().unwrap();
		openapi.paths.insert(path.to_string(), Item(item));
	}

	fn add_schema_impl(&mut self, name : String, mut schema : OpenapiSchema)
	{
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
	
	pub fn add_schema<T : OpenapiType>(&mut self) -> ReferenceOr<Schema>
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
