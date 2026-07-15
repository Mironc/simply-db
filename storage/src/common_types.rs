use std::collections::HashMap;

use crate::row::Row;

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    fields: Vec<(String, FieldType)>,
}
impl Schema {
    pub fn new(fields: Vec<(String, FieldType)>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &Vec<(String, FieldType)> {
        &self.fields
    }
}

pub trait RowCheckable: std::fmt::Debug {
    fn check(&self, row: &Row) -> bool;
}
#[derive(Debug)]
pub enum FieldModifier {
    PrimaryKey,
    NotNull,
    Default(DataValue),
    AutoIncrement,
    Unique,
    Check(Box<dyn RowCheckable>),
}
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub struct FieldType {
    data_type: DataType,
    is_nullable: bool,
    is_unique: bool,
    //default: Option<DataValue>,
}

impl FieldType {
    pub fn new(data_type: DataType, modifiers: Vec<FieldModifier>) -> Self {
        let default = modifiers.iter().find_map(|x| {
            if let FieldModifier::Default(d) = x {
                Some(d)
            } else {
                None
            }
        });
        let is_unique = modifiers
            .iter()
            .any(|x| matches!(x, FieldModifier::Unique | FieldModifier::PrimaryKey));
        let is_nullable = !modifiers.iter().any(|x| {
            matches!(
                x,
                FieldModifier::NotNull | FieldModifier::Unique | FieldModifier::PrimaryKey
            )
        });
        Self {
            data_type,
            is_nullable,
            is_unique,
        }
    }

    pub fn data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn is_nullable(&self) -> bool {
        self.is_nullable
    }

    pub fn is_unique(&self) -> bool {
        self.is_unique
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaValue {
    fields: HashMap<String, DataValue>,
}
impl SchemaValue {
    pub fn new(fields: HashMap<String, DataValue>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &HashMap<String, DataValue> {
        &self.fields
    }

    pub fn fields_mut(&mut self) -> &mut HashMap<String, DataValue> {
        &mut self.fields
    }
    pub fn validate(&self, schema: &Schema) -> bool {
        schema.fields.iter().all(|(field_name, field_type)| {
            if let Some(data_type) = self.fields.get(field_name) {
                data_type.validate(&field_type.data_type, field_type.is_nullable)
            } else {
                if field_type.is_nullable() {
                    return true;
                }
                false
            }
        }) && self
            .fields
            .iter()
            .all(|(field_name, _)| schema.fields.iter().any(|x| &x.0 == field_name))
    }
}
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Scalar(ScalarValue),
    Struct(HashMap<String, DataValue>),
    Null,
}
impl DataValue {
    pub fn validate(&self, data_type: &DataType, is_nullable: bool) -> bool {
        match self {
            DataValue::Scalar(scalar_value) => {
                if let DataType::Scalar(t) = data_type {
                    scalar_value.scalar_type() == *t
                } else {
                    false
                }
            }
            DataValue::Struct(hash_map) => {
                if let DataType::Struct(t) = data_type {
                    t.iter().all(|(field_name, field_type)| {
                        if let Some(field_value) = hash_map.get(field_name) {
                            field_value.validate(field_type, false)
                        } else {
                            false
                        }
                    }) && hash_map
                        .iter()
                        .all(|(field_name, _)| t.contains_key(field_name))
                } else {
                    false
                }
            }
            DataValue::Null => is_nullable,
        }
    }
}
impl std::fmt::Display for DataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataValue::Scalar(scalar_value) => write!(f, "{}", scalar_value),
            DataValue::Struct(_) => {
                write!(f, "struct") // That's it because right now I dont really support structs
            }
            DataValue::Null => write!(f, "NULL"),
        }
    }
}
// impl serde::Serialize for DataValue {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         match self {
//             DataValue::Scalar(scalar_value) => ,
//             DataValue::Struct(hash_map) => todo!(),// holding it for future
//             DataValue::Null => todo!(),
//         }
//     }
// }
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Scalar(ScalarType),
    Struct(HashMap<String, DataType>),
}
impl DataType {
    pub fn new_struct(struct_data: HashMap<String, DataType>) -> Self {
        Self::Struct(struct_data)
    }
}
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Int,
    Float,
    Bool,
    Text,
}
impl ScalarType {
    pub fn from_str(from: &str) -> Option<Self> {
        match from {
            "INT" => Some(ScalarType::Int),
            "FLOAT" => Some(ScalarType::Float),
            "BOOLEAN" => Some(ScalarType::Bool),
            "TEXT" => Some(ScalarType::Text),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    Int(i32),
    Float(f32),
    Bool(bool),
    Text(String),
}

impl ScalarValue {
    pub fn scalar_type(&self) -> ScalarType {
        match self {
            ScalarValue::Int(_) => ScalarType::Int,
            ScalarValue::Bool(_) => ScalarType::Bool,
            ScalarValue::Text(_) => ScalarType::Text,
            ScalarValue::Float(_) => ScalarType::Float,
        }
    }
    pub fn add(lhs: &Self, rhs: &Self) -> Option<Self> {
        Some(match (lhs, rhs) {
            (ScalarValue::Int(lhs), ScalarValue::Int(rhs)) => ScalarValue::Int(lhs + rhs),
            (ScalarValue::Int(lhs), ScalarValue::Float(rhs)) => {
                ScalarValue::Float(*lhs as f32 + rhs)
            }
            (ScalarValue::Float(lhs), ScalarValue::Int(rhs)) => {
                ScalarValue::Float(lhs + *rhs as f32)
            }
            (ScalarValue::Float(lhs), ScalarValue::Float(rhs)) => ScalarValue::Float(lhs + rhs),
            _ => return None,
        })
    }
    pub fn subtract(lhs: &Self, rhs: &Self) -> Option<Self> {
        Some(match (lhs, rhs) {
            (ScalarValue::Int(lhs), ScalarValue::Int(rhs)) => ScalarValue::Int(lhs - rhs),
            (ScalarValue::Int(lhs), ScalarValue::Float(rhs)) => {
                ScalarValue::Float(*lhs as f32 - rhs)
            }
            (ScalarValue::Float(lhs), ScalarValue::Int(rhs)) => {
                ScalarValue::Float(lhs - *rhs as f32)
            }
            (ScalarValue::Float(lhs), ScalarValue::Float(rhs)) => ScalarValue::Float(lhs - rhs),
            _ => return None,
        })
    }
    pub fn multiply(lhs: &Self, rhs: &Self) -> Option<Self> {
        Some(match (lhs, rhs) {
            (ScalarValue::Int(lhs), ScalarValue::Int(rhs)) => ScalarValue::Int(lhs * rhs),
            (ScalarValue::Int(lhs), ScalarValue::Float(rhs)) => {
                ScalarValue::Float(*lhs as f32 * rhs)
            }
            (ScalarValue::Float(lhs), ScalarValue::Int(rhs)) => {
                ScalarValue::Float(lhs * *rhs as f32)
            }
            (ScalarValue::Float(lhs), ScalarValue::Float(rhs)) => ScalarValue::Float(lhs * rhs),
            _ => return None,
        })
    }
    pub fn divide(lhs: &Self, rhs: &Self) -> Option<Self> {
        Some(match (lhs, rhs) {
            (ScalarValue::Int(lhs), ScalarValue::Int(rhs)) => ScalarValue::Int(lhs / rhs),
            (ScalarValue::Int(lhs), ScalarValue::Float(rhs)) => {
                ScalarValue::Float(*lhs as f32 / rhs)
            }
            (ScalarValue::Float(lhs), ScalarValue::Int(rhs)) => {
                ScalarValue::Float(lhs / *rhs as f32)
            }
            (ScalarValue::Float(lhs), ScalarValue::Float(rhs)) => ScalarValue::Float(lhs / rhs),
            _ => return None,
        })
    }
    pub fn modulo(lhs: &Self, rhs: &Self) -> Option<Self> {
        Some(match (lhs, rhs) {
            (ScalarValue::Int(lhs), ScalarValue::Int(rhs)) => ScalarValue::Int(lhs % rhs),
            (ScalarValue::Int(lhs), ScalarValue::Float(rhs)) => {
                ScalarValue::Float(*lhs as f32 % rhs)
            }
            (ScalarValue::Float(lhs), ScalarValue::Int(rhs)) => {
                ScalarValue::Float(lhs % *rhs as f32)
            }
            (ScalarValue::Float(lhs), ScalarValue::Float(rhs)) => ScalarValue::Float(lhs % rhs),
            _ => return None,
        })
    }
}
impl std::fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarValue::Int(i) => write!(f, "{}", i),
            ScalarValue::Float(v) => write!(f, "{}", v),
            ScalarValue::Bool(b) => write!(f, "{}", b),
            ScalarValue::Text(t) => write!(f, "{}", t),
        }
    }
}

/// Simple way to create hashmap
///
/// Example:
/// ```
/// # use storage::hashmap;
/// let map = hashmap!("Key" => "Value");
/// assert_eq!(map.get("Key"), Some(&"Value"));
/// ```
#[macro_export]
macro_rules! hashmap {
    [$($key:expr => $value:expr), +] => {{
        use std::collections::HashMap;
        let mut map = HashMap::new();
        $(map.insert($key,$value);)+
        map
    }};
}
/// Short way to create DataValue from ScalarValue
///
/// Example:
/// ```
/// # use storage::scalar;
/// # use storage::common_types::{ScalarValue, DataValue};
/// let value = scalar!(Int(25));
/// assert_eq!(value, DataValue::Scalar(ScalarValue::Int(25)));
/// ```
#[macro_export]
macro_rules! scalar {
    ($variant:ident($variant_value:expr)) => {
        storage::common_types::DataValue::Scalar(storage::common_types::ScalarValue::$variant(
            $variant_value.into(),
        ))
    };
}
/// Simple way to create DataType from ScalarType
///
/// Example:
/// ```
/// # use storage::scalar_type;
/// # use storage::common_types::{ScalarType, DataType};
/// let value = scalar_type!(Int);
/// assert_eq!(value, DataType::Scalar(ScalarType::Int));
/// ```
#[macro_export]
macro_rules! scalar_type {
    ($variant:ident) => {
        storage::common_types::DataType::Scalar(storage::common_types::ScalarType::$variant)
    };
}
#[cfg(test)]
mod tests {
    use crate::{self as simply_db, common_types::FieldModifier};
    use simply_db::common_types::{
        DataType, DataValue, FieldType, ScalarType, ScalarValue, Schema, SchemaValue,
    };
    use std::collections::HashMap;

    #[test]
    fn scalar_value_validation() {
        // Test int scalar validation
        let int_value = ScalarValue::Int(42);
        assert_eq!(int_value.scalar_type(), ScalarType::Int);

        // Test float scalar validation
        let float_value = ScalarValue::Float(3.14);
        assert_eq!(float_value.scalar_type(), ScalarType::Float);

        // Test bool scalar validation
        let bool_value = ScalarValue::Bool(true);
        assert_eq!(bool_value.scalar_type(), ScalarType::Bool);

        // Test text scalar validation
        let text_value = ScalarValue::Text("hello".to_string());
        assert_eq!(text_value.scalar_type(), ScalarType::Text);
    }

    #[test]
    fn scalar_type_from_str() {
        // Test valid type strings
        assert_eq!(ScalarType::from_str("INT"), Some(ScalarType::Int));
        assert_eq!(ScalarType::from_str("FLOAT"), Some(ScalarType::Float));
        assert_eq!(ScalarType::from_str("BOOLEAN"), Some(ScalarType::Bool));
        assert_eq!(ScalarType::from_str("TEXT"), Some(ScalarType::Text));

        // Test invalid type strings
        assert_eq!(ScalarType::from_str("UNKNOWN"), None);
    }

    #[test]
    fn struct_data_validation() {
        // Test simple struct validation
        let mut data = HashMap::new();
        data.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("John".to_string())),
        );
        data.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));

        let value = DataValue::Struct(data.clone());

        let mut fields = HashMap::new();
        fields.insert("name".to_string(), DataType::Scalar(ScalarType::Text));
        fields.insert("age".to_string(), DataType::Scalar(ScalarType::Int));
        // Test validation against expected type
        let expected_type = DataType::new_struct(fields);
        // This should validate as struct with proper field types
        assert!(value.validate(&expected_type, false));
    }

    #[test]
    fn schema_value_validation() {
        // Test schema value validation with valid data
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
        ));

        let schema = Schema::new(schema_fields);

        let mut data_fields = HashMap::new();
        data_fields.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("John".to_string())),
        );

        let data_value = SchemaValue::new(data_fields);

        // Test valid validation
        assert!(data_value.validate(&schema));

        // Test invalid validation (missing field)
        let mut data_fields_missing = HashMap::new();
        data_fields_missing.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(25)));

        let data_value_missing = SchemaValue::new(data_fields_missing);

        // This should fail validation since "name" field is missing
        assert!(!data_value_missing.validate(&schema));
    }

    #[test]
    fn nullable_field_validation() {
        // Test nullable field validation
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
        ));

        let schema = Schema::new(schema_fields);

        // Test valid validation with nullable field
        let mut data_fields = HashMap::new();
        data_fields.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("John".to_string())),
        );

        let data_value = SchemaValue::new(data_fields);

        // Test valid validation
        assert!(data_value.validate(&schema));

        // Test validation when field is nullable and value is null
        let mut data_fields_nullable = HashMap::new();
        data_fields_nullable.insert("name".to_string(), DataValue::Null);

        let data_value_nullable = SchemaValue::new(data_fields_nullable);

        // This should validate as field is nullable
        assert!(data_value_nullable.validate(&schema));
    }

    #[test]
    fn type_mismatch_validation() {
        // Test type mismatch validation
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
        ));

        let schema = Schema::new(schema_fields);

        // Test invalid validation (wrong type)
        let mut data_fields = HashMap::new();
        data_fields.insert("name".to_string(), DataValue::Scalar(ScalarValue::Int(42)));

        let data_value = SchemaValue::new(data_fields);

        // This should fail validation since type doesn't match
        assert!(!data_value.validate(&schema));
    }
    #[test]
    fn schema_value_excess_fields_validation() {
        // Test schema with only one field
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
        ));

        let schema = Schema::new(schema_fields);

        // Create schema value with excess fields (more than schema defines)
        let mut data_fields = HashMap::new();
        data_fields.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("John".to_string())),
        );
        data_fields.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));
        data_fields.insert(
            "city".to_string(),
            DataValue::Scalar(ScalarValue::Text("New York".to_string())),
        );

        let data_value = SchemaValue::new(data_fields);

        // This should fail validation since we have excess fields
        // The schema only defines "name" field, so "age" and "city" are invalid
        assert!(!data_value.validate(&schema));
    }

    #[test]
    fn schema_value_excess_fields_with_nullable_validation() {
        // Test schema with only one field (nullable)
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(
                DataType::Scalar(ScalarType::Text),
                vec![FieldModifier::NotNull],
            ),
        ));

        let schema = Schema::new(schema_fields);

        // Create schema value with excess fields (more than schema defines)
        let mut data_fields = HashMap::new();
        data_fields.insert(
            "name".to_string(),
            DataValue::Scalar(ScalarValue::Text("John".to_string())),
        );
        data_fields.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));

        let data_value = SchemaValue::new(data_fields);

        // This should fail validation since we have excess fields
        // The schema only defines "name" field, so "age" is invalid
        assert!(!data_value.validate(&schema));
    }

    #[test]
    fn schema_value_excess_fields_with_missing_required() {
        // Test schema with required field
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(DataType::Scalar(ScalarType::Text), vec![]),
        ));

        let schema = Schema::new(schema_fields);

        // Create schema value with missing required field
        let mut data_fields = HashMap::new();
        data_fields.insert("age".to_string(), DataValue::Scalar(ScalarValue::Int(30)));

        let data_value = SchemaValue::new(data_fields);

        // This should fail validation since required "name" field is missing
        assert!(!data_value.validate(&schema));
    }
}
