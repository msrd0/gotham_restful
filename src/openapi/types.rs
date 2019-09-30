use indexmap::IndexMap;
use openapiv3::{
	ArrayType, IntegerType, NumberType, ObjectType, ReferenceOr::Item, Schema, SchemaData, SchemaKind, StringType, Type
};
use serde::Serialize;


pub trait OpenapiType : Serialize
{
	fn schema_name() -> Option<String>
	{
		None
	}
	
	fn to_schema() -> SchemaKind;
}

impl OpenapiType for ()
{
	fn to_schema() -> SchemaKind
	{
		SchemaKind::Type(Type::Object(ObjectType::default()))
	}
}

impl OpenapiType for bool
{
	fn to_schema() -> SchemaKind
	{
		SchemaKind::Type(Type::Boolean{})
	}
}

macro_rules! int_types {
	($($int_ty:ty),*) => {$(
		impl OpenapiType for $int_ty
		{
			fn to_schema() -> SchemaKind
			{
				SchemaKind::Type(Type::Integer(IntegerType::default()))
			}
		}
	)*}
}

int_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128);

macro_rules! num_types {
	($($num_ty:ty),*) => {$(
		impl OpenapiType for $num_ty
		{
			fn to_schema() -> SchemaKind
			{
				SchemaKind::Type(Type::Number(NumberType::default()))
			}
		}
	)*}
}

num_types!(f32, f64);

macro_rules! str_types {
	($($str_ty:ty),*) => {$(
		impl OpenapiType for $str_ty
		{
			fn to_schema() -> SchemaKind
			{
				SchemaKind::Type(Type::String(StringType::default()))
			}
		}
	)*}
}

str_types!(String, &str);

impl<T : OpenapiType> OpenapiType for Vec<T>
{
	fn schema_name() -> Option<String>
	{
		T::schema_name().map(|name| format!("{}Array", name))
	}
	
	fn to_schema() -> SchemaKind
	{
		SchemaKind::Type(Type::Array(ArrayType {
			items: Item(Box::new(Schema {
				schema_data: SchemaData::default(),
				schema_kind: T::to_schema()
			})),
			min_items: None,
			max_items: None,
			unique_items: false
		}))
	}
}
