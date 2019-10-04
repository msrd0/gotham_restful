#[cfg(feature = "openapi")]
pub mod openapi
{
	pub use indexmap::IndexMap;
	pub use openapiv3::{ObjectType, ReferenceOr, Schema, SchemaData, SchemaKind, StringType, Type, VariantOrUnknownOrEmpty};
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
