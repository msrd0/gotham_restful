#[cfg(feature = "openapi")]
pub mod openapi
{
	pub use indexmap::IndexMap;
	pub use openapiv3::{ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, StringType, Type, VariantOrUnknownOrEmpty};
}

#[cfg(not(feature = "openapi"))]
#[macro_export]
macro_rules! rest_struct {
	($struct_name:ident { $($field_id:ident : $field_ty:ty),* }) => {
		#[derive(serde::Deserialize, serde::Serialize)]
		pub struct $struct_name
		{
			$($field_id : $field_ty),*
		}
	}
}

#[cfg(feature = "openapi")]
#[macro_export]
macro_rules! rest_struct {
	($struct_name:ident { $($field_id:ident : $field_ty:ty),* }) => {
		#[derive(serde::Deserialize, serde::Serialize)]
		struct $struct_name
		{
			$($field_id : $field_ty),*
		}

		impl ::gotham_restful::OpenapiType for $struct_name
		{
			fn to_schema() -> ::gotham_restful::OpenapiSchema
			{
				use ::gotham_restful::{helper::openapi::*, OpenapiSchema};

				let mut properties : IndexMap<String, ReferenceOr<Box<Schema>>> = IndexMap::new();
				let mut required : Vec<String> = Vec::new();
				let mut dependencies : IndexMap<String, OpenapiSchema> = IndexMap::new();

				$(
					{
						let mut schema = <$field_ty>::to_schema();
						
						if schema.nullable
						{
							schema.nullable = false;
							schema.name = schema.name.map(|name|
														  if name.ends_with("OrNull") {
															  name[..(name.len()-6)].to_string()
														  } else { name });
						}
						else
						{
							required.push(stringify!($field_id).to_string());
						}
						
						if let Some(name) = schema.name.clone()
						{
							properties.insert(
								stringify!($field_id).to_string(),
								ReferenceOr::Reference { reference: format!("#/components/schemas/{}", name) }
							);
							dependencies.insert(name, schema);
						}
						else
						{
							properties.insert(
								stringify!($field_id).to_string(),
								ReferenceOr::Item(Box::new(<$field_ty>::to_schema().to_schema()))
							);
						}
					}
				)*
				
				let schema = SchemaKind::Type(Type::Object(ObjectType {
					properties,
					required,
					additional_properties: None,
					min_properties: None,
					max_properties: None
				}));

				OpenapiSchema {
					name: Some(stringify!($struct_name).to_string()),
					nullable: false,
					schema,
					dependencies
				}
			}
		}
	}
}

#[macro_export]
macro_rules! rest_resource {
	($res_name:ident, $route:ident => $setup:block) => {
		pub struct $res_name;

		impl ::gotham_restful::Resource for $res_name
		{
			fn name() -> String
			{
				stringify!($res_name).to_string()
			}

			fn setup<D : ::gotham_restful::DrawResourceRoutes>(mut $route : D) $setup
		}
	}
}
