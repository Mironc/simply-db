use structures::VecMap;

use crate::{
    common_types::{DataValue, ScalarType},
    row::Row,
};

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
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    fields: VecMap<String, FieldType>,
}

impl Schema {
    pub fn new(fields: VecMap<String, FieldType>) -> Self {
        Self { fields }
    }

    pub fn fields(&self) -> &VecMap<String, FieldType> {
        &self.fields
    }

    pub fn build_index_map(&self, field_names: &[String]) -> Option<Vec<Option<usize>>> {
        let schema_len = self.fields.len();
        let mut index_map = vec![None; schema_len];

        for (src_idx, name) in field_names.iter().enumerate() {
            match self.fields.get_index(name) {
                Some(target_idx) => {
                    index_map[target_idx] = Some(src_idx);
                }
                None => return None,
            }
        }

        Some(index_map)
    }

    pub fn validate(&self, index_map: &[Option<usize>], row: &Vec<DataValue>) -> bool {
        for (target_idx, field) in self.fields.values().enumerate() {
            match index_map[target_idx] {
                Some(src_idx) => {
                    if let Some(input_value) = row.get(src_idx) {
                        match input_value {
                            DataValue::Scalar(s_val) => {
                                if s_val.scalar_type() != field.data_type() {
                                    return false;
                                }
                            }
                            DataValue::Null => {
                                if !field.is_nullable() {
                                    return false;
                                }
                            }
                        }
                    } else {
                        // The index map pointed to a source index out of bounds of the actual row
                        return false;
                    }
                }
                None => {
                    // If field is not present and it isn't nullable validation fails
                    if !field.is_nullable() {
                        return false;
                    }
                }
            }
        }
        true
    }
    pub fn order_row(
        &self,
        index_map: &[Option<usize>],
        values: &mut Vec<DataValue>,
        temp_buffer: &mut Vec<DataValue>,
    ) {
        // Move values into temporary buffer
        std::mem::swap(values, temp_buffer);
        values.clear();

        for &source_idx in index_map {
            match source_idx {
                Some(idx) => {
                    // From temporary buffer push values from source to appropriate index
                    let val = std::mem::replace(&mut temp_buffer[idx], DataValue::Null);
                    values.push(val);
                }
                None => {
                    // If source doesn't have needed field, pushes null value
                    values.push(DataValue::Null);
                }
            }
        }

        temp_buffer.clear();
    }
}

#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldType {
    data_type: ScalarType,
    is_nullable: bool,
    is_unique: bool,
}

impl FieldType {
    pub fn new(data_type: ScalarType, modifiers: Vec<FieldModifier>) -> Self {
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

    pub fn data_type(&self) -> ScalarType {
        self.data_type
    }

    pub fn is_nullable(&self) -> bool {
        self.is_nullable
    }

    pub fn is_unique(&self) -> bool {
        self.is_unique
    }
}
#[cfg(test)]
mod tests {
    use crate as storage;
    use crate::{
        common_types::{DataValue, ScalarType, ScalarValue},
        scalar,
        schema::{FieldType, Schema},
    };

    #[test]
    fn row_validation() {
        // Test schema value validation with valid data
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into());

        let mut data_fields = Vec::new();
        data_fields.push(DataValue::Scalar(ScalarValue::Text("John".to_string())));

        let index_map = schema.build_index_map(&["name".to_owned()]).unwrap();
        // Test valid validation
        assert!(schema.validate(&index_map, &data_fields));

        // Test invalid validation (unknown field), catched by index mapping
        let mut data_fields_missing = Vec::new();
        data_fields_missing.push(DataValue::Scalar(ScalarValue::Int(25)));
        assert_eq!(schema.build_index_map(&["age".to_owned()]), None);
    }

    #[test]
    fn nullable_field_validation() {
        // Test nullable field validation
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into());

        // Test valid validation with nullable field
        let mut data_fields = Vec::new();
        data_fields.push(DataValue::Scalar(ScalarValue::Text("John".to_string())));

        let index_map = schema.build_index_map(&["name".to_owned()]).unwrap();
        // Test valid validation
        assert!(schema.validate(&index_map, &data_fields));

        // Test validation when field is nullable and value is null
        let mut data_fields_nullable = Vec::new();
        data_fields_nullable.push(DataValue::Null);
        let index_map = schema.build_index_map(&["name".to_owned()]).unwrap();

        // This should validate as field is nullable
        assert!(schema.validate(&index_map, &data_fields_nullable));
    }

    #[test]
    fn type_mismatch_validation() {
        // Test type mismatch validation
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into());

        // Test invalid validation (wrong type)
        let mut data_fields = Vec::new();
        data_fields.push(scalar!(Int(42)));
        let index_map = schema.build_index_map(&["name".to_owned()]).unwrap();

        // This should fail validation since type doesn't match
        assert!(!schema.validate(&index_map, &data_fields));
    }
    #[test]
    fn schema_value_excess_fields_validation() {
        // Test schema with only one field
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into());

        // Create row with excess fields (more than schema defines)
        let mut data_fields = Vec::new();
        data_fields.push(scalar!(Int(30)));
        data_fields.push(scalar!(Text("John".to_owned())));
        data_fields.push(scalar!(Text("New York".to_owned())));
        // Excessive fields catched by index mapping
        assert_eq!(
            schema.build_index_map(&["name".to_owned(), "age".to_owned(), "city".to_owned()]),
            None
        );
    }
    #[test]
    fn row_ordering() {
        // Ordering in schema is "name, city, age"
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));
        schema_fields.push(("city".to_string(), FieldType::new(ScalarType::Text, vec![])));
        schema_fields.push(("age".to_string(), FieldType::new(ScalarType::Int, vec![])));

        let schema = Schema::new(schema_fields.into());

        // Ordering in row is "age, name, city"
        let mut data_fields = Vec::new();
        data_fields.push(scalar!(Int(30)));
        data_fields.push(scalar!(Text("John".to_owned())));
        data_fields.push(scalar!(Text("New York".to_owned())));
        let index_map = schema
            .build_index_map(&["age".to_owned(), "name".to_owned(), "city".to_owned()])
            .unwrap();
        let mut temp = Vec::new();
        schema.order_row(&index_map, &mut data_fields, &mut temp);
        assert_eq!(
            data_fields,
            [
                scalar!(Text("John".to_owned())),
                scalar!(Text("New York".to_owned())),
                scalar!(Int(30)),
            ]
        )
    }
}
