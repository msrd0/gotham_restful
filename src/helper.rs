#[cfg(feature = "openapi")]
pub mod openapi
{
	pub use indexmap::IndexMap;
	pub use openapiv3::{ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, Type};
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
			fn schema_name() -> Option<String>
			{
				Some(stringify!($struct_name).to_string())
			}
			
			fn to_schema() -> ::gotham_restful::helper::openapi::SchemaKind
			{
				use ::gotham_restful::helper::openapi::*;

				let mut properties : IndexMap<String, ReferenceOr<Box<Schema>>> = IndexMap::new();
				let mut required : Vec<String> = Vec::new();

				$(
					properties.insert(
						stringify!($field_id).to_string(),
						ReferenceOr::Item(Box::new(Schema {
							schema_data: SchemaData::default(),
							schema_kind: <$field_ty>::to_schema()
						}))
					);
				)*
				
				SchemaKind::Type(Type::Object(ObjectType {
					properties,
					required,
					additional_properties: None,
					min_properties: None,
					max_properties: None
				}))
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
