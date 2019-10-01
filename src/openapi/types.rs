#[cfg(feature = "chrono")]
use chrono::{
	Date, DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, Utc
};
use indexmap::IndexMap;
use openapiv3::{
	ArrayType, IntegerType, NumberType, ObjectType, ReferenceOr::Item, ReferenceOr::Reference, Schema,
	SchemaData, SchemaKind, StringFormat, StringType, Type, VariantOrUnknownOrEmpty
};

#[derive(Debug, Clone, PartialEq)]
pub struct OpenapiSchema
{
	/// The name of this schema. If it is None, the schema will be inlined.
	pub name : Option<String>,
	pub nullable : bool,
	pub schema : SchemaKind,
	pub dependencies : IndexMap<String, OpenapiSchema>
}

impl OpenapiSchema
{
	pub fn new(schema : SchemaKind) -> Self
	{
		Self {
			name: None,
			nullable: false,
			schema,
			dependencies: IndexMap::new()
		}
	}

	pub fn to_schema(self) -> Schema
	{
		Schema {
			schema_data: SchemaData {
				nullable: self.nullable,
				read_only: false,
				write_only: false,
				deprecated: false,
				external_docs: None,
				example: None,
				title: self.name,
				description: None,
				discriminator: None,
				default: None
			},
			schema_kind: self.schema
		}
	}
}

pub trait OpenapiType
{
	fn to_schema() -> OpenapiSchema;
}

impl OpenapiType for ()
{
	fn to_schema() -> OpenapiSchema
	{
		OpenapiSchema::new(SchemaKind::Type(Type::Object(ObjectType::default())))
	}
}

impl OpenapiType for bool
{
	fn to_schema() -> OpenapiSchema
	{
		OpenapiSchema::new(SchemaKind::Type(Type::Boolean{}))
	}
}

macro_rules! int_types {
	($($int_ty:ty),*) => {$(
		impl OpenapiType for $int_ty
		{
			fn to_schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Integer(IntegerType::default())))
			}
		}
	)*}
}

int_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128);

macro_rules! num_types {
	($($num_ty:ty),*) => {$(
		impl OpenapiType for $num_ty
		{
			fn to_schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::Number(NumberType::default())))
			}
		}
	)*}
}

num_types!(f32, f64);

macro_rules! str_types {
	($($str_ty:ty),*) => {$(
		impl OpenapiType for $str_ty
		{
			fn to_schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType::default())))
			}
		}
	)*};

	(format = $format:ident, $($str_ty:ty),*) => {$(
		impl OpenapiType  for $str_ty
		{
			fn to_schema() -> OpenapiSchema
			{
				OpenapiSchema::new(SchemaKind::Type(Type::String(StringType {
					format: VariantOrUnknownOrEmpty::Item(StringFormat::$format),
					pattern: None,
					enumeration: Vec::new()
				})))
			}
		}
	)*};
}

str_types!(String, &str);

impl<T : OpenapiType> OpenapiType for Option<T>
{
	fn to_schema() -> OpenapiSchema
	{
		let schema = T::to_schema();
		let mut dependencies : IndexMap<String, OpenapiSchema> = IndexMap::new();
		let refor = if let Some(name) = schema.name.clone()
		{
			let reference = Reference { reference: format!("#/components/schemas/{}", name) };
			dependencies.insert(name, schema);
			reference
		}
		else
		{
			Item(schema.to_schema())
		};
		
		OpenapiSchema {
			nullable: true,
			name: None,
			schema: SchemaKind::AllOf { all_of: vec![refor] },
			dependencies
		}
	}
}

impl<T : OpenapiType> OpenapiType for Vec<T>
{
	fn to_schema() -> OpenapiSchema
	{
		let schema = T::to_schema();
		let mut dependencies : IndexMap<String, OpenapiSchema> = IndexMap::new();

		let items = if let Some(name) = schema.name.clone()
		{
			let reference = Reference { reference: format!("#/components/schemas/{}", name) };
			dependencies.insert(name, schema);
			reference
		}
		else
		{
			Item(Box::new(schema.to_schema()))
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

#[cfg(feature = "chrono")]
str_types!(format = Date, Date<FixedOffset>, Date<Local>, Date<Utc>, NaiveDate);
#[cfg(feature = "chrono")]
str_types!(format = DateTime, DateTime<FixedOffset>, DateTime<Local>, DateTime<Utc>, NaiveDateTime);
