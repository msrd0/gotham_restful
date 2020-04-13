#[cfg(feature = "chrono")]
use chrono::{
	Date, DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, Utc
};
use indexmap::IndexMap;
use openapiv3::{
	AdditionalProperties, ArrayType, IntegerType, NumberFormat, NumberType, ObjectType, ReferenceOr::Item,
	ReferenceOr::Reference, Schema, SchemaData, SchemaKind, StringType, Type, VariantOrUnknownOrEmpty
};
#[cfg(feature = "uuid")]
use uuid::Uuid;
use std::collections::{BTreeSet, HashSet};

/**
This struct needs to be available for every type that can be part of an OpenAPI Spec. It is
already implemented for primitive types, String, Vec, Option and the like. To have it available
for your type, simply derive from [`OpenapiType`].

[`OpenapiType`]: trait.OpenapiType.html
*/
#[derive(Debug, Clone, PartialEq)]
pub struct OpenapiSchema
{
	/// The name of this schema. If it is None, the schema will be inlined.
	pub name : Option<String>,
	/// Whether this particular schema is nullable. Note that there is no guarantee that this will
	/// make it into the final specification, it might just be interpreted as a hint to make it
	/// an optional parameter.
	pub nullable : bool,
	/// The actual OpenAPI schema.
	pub schema : SchemaKind,
	/// Other schemas that this schema depends on. They will be included in the final OpenAPI Spec
	/// along with this schema.
	pub dependencies : IndexMap<String, OpenapiSchema>
}

impl OpenapiSchema
{
	/// Create a new schema that has no name.
	pub fn new(schema : SchemaKind) -> Self
	{
		Self {
			name: None,
			nullable: false,
			schema,
			dependencies: IndexMap::new()
		}
	}
	
	/// Convert this schema to an `openapiv3::Schema` that can be serialized to the OpenAPI Spec.
	pub fn into_schema(self) -> Schema
	{
		Schema {
			schema_data: SchemaData {
				nullable: self.nullable,
				title: self.name,
				..Default::default()
			},
			schema_kind: self.schema
		}
	}
}

/**
This trait needs to be implemented by every type that is being used in the OpenAPI Spec. It gives
access to the [`OpenapiSchema`] of this type. It is provided for primitive types, String and the
like. For use on your own types, there is a derive macro:

```
# #[macro_use] extern crate gotham_restful_derive;
#
#[derive(OpenapiType)]
struct MyResponse {
	message: String
}
```

[`OpenapiSchema`]: struct.OpenapiSchema.html
*/
pub trait OpenapiType
{
	fn schema() -> OpenapiSchema;
}

impl OpenapiType for ()
{
	fn schema() -> OpenapiSchema
	{
		OpenapiSchema::new(SchemaKind::Type(Type::Object(ObjectType {
			additional_properties: Some(AdditionalProperties::Any(false)),
			..Default::default()
		})))
	}
}

impl OpenapiType for bool
{
	fn schema() -> OpenapiSchema
	{
		OpenapiSchema::new(SchemaKind::Type(Type::Boolean{}))
	}
}

macro_rules! int_types {
	($($int_ty:ty),*) => {$(
		impl OpenapiType for $int_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Integer(IntegerType::default())))
			}
		}
	)*};
	
	(unsigned $($int_ty:ty),*) => {$(
		impl OpenapiType for $int_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Integer(IntegerType {
					minimum: Some(0),
					..Default::default()
				})))
			}
		}
	)*};
	
	(bits = $bits:expr, $($int_ty:ty),*) => {$(
		impl OpenapiType for $int_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Integer(IntegerType {
					format: VariantOrUnknownOrEmpty::Unknown(format!("int{}", $bits)),
					..Default::default()
				})))
			}
		}
	)*};
	
	(unsigned bits = $bits:expr, $($int_ty:ty),*) => {$(
		impl OpenapiType for $int_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Integer(IntegerType {
					format: VariantOrUnknownOrEmpty::Unknown(format!("int{}", $bits)),
					minimum: Some(0),
					..Default::default()
				})))
			}
		}
	)*};
}

int_types!(isize);
int_types!(unsigned usize);
int_types!(bits = 8, i8);
int_types!(unsigned bits = 8, u8);
int_types!(bits = 16, i16);
int_types!(unsigned bits = 16, u16);
int_types!(bits = 32, i32);
int_types!(unsigned bits = 32, u32);
int_types!(bits = 64, i64);
int_types!(unsigned bits = 64, u64);
int_types!(bits = 128, i128);
int_types!(unsigned bits = 128, u128);

macro_rules! num_types {
	($($num_ty:ty = $num_fmt:ident),*) => {$(
		impl OpenapiType for $num_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Number(NumberType {
					format: VariantOrUnknownOrEmpty::Item(NumberFormat::$num_fmt),
					..Default::default()
				})))
			}
		}
	)*}
}

num_types!(f32 = Float, f64 = Double);

macro_rules! str_types {
	($($str_ty:ty),*) => {$(
		impl OpenapiType for $str_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType::default())))
			}
		}
	)*};

	(format = $format:ident, $($str_ty:ty),*) => {$(
		impl OpenapiType  for $str_ty
		{
			fn schema() -> OpenapiSchema
			{
				use openapiv3::StringFormat;
				
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
					format: VariantOrUnknownOrEmpty::Item(StringFormat::$format),
					..Default::default()
				})))
			}
		}
	)*};
	
	(format_str = $format:expr, $($str_ty:ty),*) => {$(
		impl OpenapiType  for $str_ty
		{
			fn schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
					format: VariantOrUnknownOrEmpty::Unknown($format.to_string()),
					..Default::default()
				})))
			}
		}
	)*};
}

str_types!(String, &str);

#[cfg(feature = "chrono")]
str_types!(format = Date, Date<FixedOffset>, Date<Local>, Date<Utc>, NaiveDate);
#[cfg(feature = "chrono")]
str_types!(format = DateTime, DateTime<FixedOffset>, DateTime<Local>, DateTime<Utc>, NaiveDateTime);

#[cfg(feature = "uuid")]
str_types!(format_str = "uuid", Uuid);

impl<T : OpenapiType> OpenapiType for Option<T>
{
	fn schema() -> OpenapiSchema
	{
		let schema = T::schema();
		let mut dependencies = schema.dependencies.clone();
		let schema = match schema.name.clone() {
			Some(name) => {
				let reference = Reference { reference: format!("#/components/schemas/{}", name) };
				dependencies.insert(name, schema);
				SchemaKind::AllOf { all_of: vec![reference] }
			},
			None => schema.schema
		};
		
		OpenapiSchema {
			nullable: true,
			name: None,
			schema,
			dependencies
		}
	}
}

impl<T : OpenapiType> OpenapiType for Vec<T>
{
	fn schema() -> OpenapiSchema
	{
		let schema = T::schema();
		let mut dependencies = schema.dependencies.clone();
		
		let items = match schema.name.clone()
		{
			Some(name) => {
				let reference = Reference { reference: format!("#/components/schemas/{}", name) };
				dependencies.insert(name, schema);
				reference
			},
			None => Item(Box::new(schema.into_schema()))
		};
		
		OpenapiSchema {
			nullable: false,
			name: None,
			schema: SchemaKind::Type(Type::Array(ArrayType {
				items,
				min_items: None,
				max_items: None,
				unique_items: false
			})),
			dependencies
		}
	}
}

impl<T : OpenapiType> OpenapiType for BTreeSet<T>
{
	fn schema() -> OpenapiSchema
	{
		<Vec<T> as OpenapiType>::schema()
	}
}

impl<T : OpenapiType> OpenapiType for HashSet<T>
{
	fn schema() -> OpenapiSchema
	{
		<Vec<T> as OpenapiType>::schema()
	}
}

impl OpenapiType for serde_json::Value
{
	fn schema() -> OpenapiSchema
	{
		OpenapiSchema {
			nullable: true,
			name: None,
			schema: SchemaKind::Any(Default::default()),
			dependencies: Default::default()
		}
	}
}


#[cfg(test)]
mod test
{
	use super::*;
	use serde_json::Value;
	
	type Unit = ();
	
	macro_rules! assert_schema {
		($ty:ident $(<$generic:ident>)* => $json:expr) => {
			paste::item! {
				#[test]
				fn [<test_schema_ $ty:snake $(_ $generic:snake)*>]()
				{
					let schema = <$ty $(<$generic>)* as OpenapiType>::schema().into_schema();
					let schema_json = serde_json::to_string(&schema).expect(&format!("Unable to serialize schema for {}", stringify!($ty)));
					assert_eq!(schema_json, $json);
				}
			}
		};
	}
	
	assert_schema!(Unit => r#"{"type":"object","additionalProperties":false}"#);
	assert_schema!(bool => r#"{"type":"boolean"}"#);
	assert_schema!(isize => r#"{"type":"integer"}"#);
	assert_schema!(usize => r#"{"type":"integer","minimum":0}"#);
	assert_schema!(i8 => r#"{"type":"integer","format":"int8"}"#);
	assert_schema!(u8 => r#"{"type":"integer","format":"int8","minimum":0}"#);
	assert_schema!(i16 => r#"{"type":"integer","format":"int16"}"#);
	assert_schema!(u16 => r#"{"type":"integer","format":"int16","minimum":0}"#);
	assert_schema!(i32 => r#"{"type":"integer","format":"int32"}"#);
	assert_schema!(u32 => r#"{"type":"integer","format":"int32","minimum":0}"#);
	assert_schema!(i64 => r#"{"type":"integer","format":"int64"}"#);
	assert_schema!(u64 => r#"{"type":"integer","format":"int64","minimum":0}"#);
	assert_schema!(i128 => r#"{"type":"integer","format":"int128"}"#);
	assert_schema!(u128 => r#"{"type":"integer","format":"int128","minimum":0}"#);
	assert_schema!(f32 => r#"{"type":"number","format":"float"}"#);
	assert_schema!(f64 => r#"{"type":"number","format":"double"}"#);
	
	assert_schema!(String => r#"{"type":"string"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(Date<FixedOffset> => r#"{"type":"string","format":"date"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(Date<Local> => r#"{"type":"string","format":"date"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(Date<Utc> => r#"{"type":"string","format":"date"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(NaiveDate => r#"{"type":"string","format":"date"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(DateTime<FixedOffset> => r#"{"type":"string","format":"date-time"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(DateTime<Local> => r#"{"type":"string","format":"date-time"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(DateTime<Utc> => r#"{"type":"string","format":"date-time"}"#);
	#[cfg(feature = "chrono")]
	assert_schema!(NaiveDateTime => r#"{"type":"string","format":"date-time"}"#);
	#[cfg(feature = "uuid")]
	assert_schema!(Uuid => r#"{"type":"string","format":"uuid"}"#);
	
	assert_schema!(Option<String> => r#"{"nullable":true,"type":"string"}"#);
	assert_schema!(Vec<String> => r#"{"type":"array","items":{"type":"string"}}"#);
	
	assert_schema!(Value => r#"{"nullable":true}"#);
}
