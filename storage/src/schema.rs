use structures::VecMap;

use crate::{
    common_types::{DataValue, ScalarType},
    row::Row,
};
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaError {
    IncompatibleModifier,
    MultipleAutoIncrement,
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
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    fields: VecMap<String, FieldType>,
    autoincrement_field: Option<usize>,
}

impl Schema {
    pub fn new(fields: VecMap<String, FieldType>) -> Result<Self, SchemaError> {
        let mut autoincrement_fields = None;
        for (i, field) in fields.values().enumerate() {
            field.validate()?;
            if field.auto_increment() {
                if autoincrement_fields.is_some() {
                    return Err(SchemaError::MultipleAutoIncrement);
                }
                autoincrement_fields = Some(i);
            }
        }
        Ok(Self {
            fields,
            autoincrement_field: autoincrement_fields,
        })
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

    pub fn validate(&self, index_map: &[Option<usize>], row: &[DataValue]) -> bool {
        for (target_idx, field) in self.fields.values().enumerate() {
            // If field is marked as AUTOINCREMENT, provided value is ignored
            if field.auto_increment {
                continue;
            }
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

        for (target_idx, (_, field)) in self.fields.iter().enumerate() {
            let source_idx = index_map[target_idx];
            if field.auto_increment() {
                match field.data_type() {
                    ScalarType::Int => {
                        // Sets as null to replace it later
                        values.push(DataValue::Null);
                    }
                    _ => unreachable!(),
                }
            } else {
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
        }

        temp_buffer.clear();
    }

    pub fn autoincrement_field(&self) -> Option<usize> {
        self.autoincrement_field
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
    auto_increment: bool,
}

impl FieldType {
    pub fn new(data_type: ScalarType, modifiers: Vec<FieldModifier>) -> Self {
        let mut is_unique = false;
        let mut is_nullable = true;
        let mut auto_increment = false;
        let mut _default_value = None;
        for modifer in modifiers.into_iter() {
            match modifer {
                FieldModifier::PrimaryKey => {
                    is_unique = true;
                    is_nullable = false;
                }
                FieldModifier::Unique => is_unique = true,
                FieldModifier::NotNull => is_nullable = false,
                FieldModifier::Default(data_value) => _default_value = Some(data_value),
                FieldModifier::AutoIncrement => auto_increment = true,
                FieldModifier::Check(_) => (),
            }
        }
        Self {
            data_type,
            is_nullable,
            is_unique,
            auto_increment,
        }
    }
    /// # Errors:
    ///
    /// - Returns `IncompatibleModifier`, if AUTOINCREMENT modifier is applied on non-integer field.
    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.auto_increment {
            match self.data_type {
                ScalarType::Int => (),
                _ => return Err(SchemaError::IncompatibleModifier),
            }
        }
        Ok(())
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

    pub fn auto_increment(&self) -> bool {
        self.auto_increment
    }
}
#[cfg(test)]
mod tests {
    use crate as storage;
    use crate::schema::FieldModifier::AutoIncrement;
    use crate::schema::{FieldModifier, SchemaError};
    use crate::{
        common_types::{DataValue, ScalarType, ScalarValue},
        scalar,
        schema::{FieldType, Schema},
    };

    #[test]
    fn incompatible_modifier_error() {
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "id".to_string(),
            FieldType::new(ScalarType::Text, vec![FieldModifier::AutoIncrement]),
        ));

        let error = Schema::new(schema_fields.into()).unwrap_err();
        assert_eq!(error, SchemaError::IncompatibleModifier);
    }

    #[test]
    fn multiple_autoincrement_error() {
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "id".to_string(),
            FieldType::new(ScalarType::Int, vec![FieldModifier::AutoIncrement]),
        ));
        schema_fields.push((
            "id1".to_string(),
            FieldType::new(ScalarType::Int, vec![FieldModifier::AutoIncrement]),
        ));

        let error = Schema::new(schema_fields.into()).unwrap_err();
        assert_eq!(error, SchemaError::MultipleAutoIncrement);
    }
    #[test]
    fn non_nullable_validation() {
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "name".to_string(),
            FieldType::new(ScalarType::Text, vec![FieldModifier::NotNull]),
        ));

        let schema = Schema::new(schema_fields.into()).unwrap();
        let index_map = schema.build_index_map(&["name".to_owned()]).unwrap();

        let mut data_fields = Vec::new();
        data_fields.push(DataValue::Scalar(ScalarValue::Text("John".to_string())));
        // Test valid validation
        assert!(schema.validate(&index_map, &data_fields));

        let mut data_fields = Vec::new();
        data_fields.push(DataValue::Null);
        // Test invalid with NULL value
        assert!(!schema.validate(&index_map, &data_fields));

        let index_map = schema.build_index_map(&[]).unwrap();
        // Test invalid without providing value
        assert!(!schema.validate(&index_map, &vec![]));
    }
    #[test]
    fn nullable_field_validation() {
        // Test nullable field validation
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into()).unwrap();

        // Test validation with nullable field
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
    fn autoincrement_field_validation() {
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "id".to_string(),
            FieldType::new(ScalarType::Int, vec![AutoIncrement]),
        ));
        let schema = Schema::new(schema_fields.into()).unwrap();
        let index_map = schema.build_index_map(&[]).unwrap();

        // Test with no data
        let empty_row = Vec::new();
        assert!(schema.validate(&index_map, &empty_row));

        let index_map = schema.build_index_map(&["id".to_string()]).unwrap();
        // Test with null
        let null_row = vec![DataValue::Null];
        assert!(schema.validate(&index_map, &null_row));

        // Test with some value
        let some_value = vec![DataValue::Scalar(ScalarValue::Int(1))];
        assert!(schema.validate(&index_map, &some_value));
    }

    #[test]
    fn type_mismatch_validation() {
        // Test type mismatch validation
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into()).unwrap();

        // Test invalid validation (wrong type)
        let mut data_fields = Vec::new();
        data_fields.push(scalar!(Int(42)));
        let index_map = schema.build_index_map(&["name".to_owned()]).unwrap();

        // This should fail validation since type doesn't match
        assert!(!schema.validate(&index_map, &data_fields));
    }

    #[test]
    fn excessive_fields_validation() {
        // Test schema with only one field
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));

        let schema = Schema::new(schema_fields.into()).unwrap();

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

        let schema = Schema::new(schema_fields.into()).unwrap();

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

    #[test]
    fn row_ordering_null_insertion() {
        // Schema with nullable field
        let mut schema_fields = Vec::new();
        schema_fields.push(("name".to_string(), FieldType::new(ScalarType::Text, vec![])));
        let schema = Schema::new(schema_fields.into()).unwrap();

        // No nulls
        let mut data_fields = Vec::new();

        let index_map = schema.build_index_map(&[]).unwrap();
        let mut temp = Vec::new();
        schema.order_row(&index_map, &mut data_fields, &mut temp);
        // Null inserted
        assert_eq!(data_fields, [DataValue::Null])
    }

    #[test]
    fn row_ordering_auto_increment_insert() {
        // schema with id
        let mut schema_fields = Vec::new();
        schema_fields.push((
            "id".to_string(),
            FieldType::new(ScalarType::Int, vec![FieldModifier::AutoIncrement]),
        ));
        let schema = Schema::new(schema_fields.into()).unwrap();

        let mut data_fields = Vec::new();

        // With no value
        let index_map = schema.build_index_map(&[]).unwrap();
        let mut temp = Vec::new();
        schema.order_row(&index_map, &mut data_fields, &mut temp);
        assert_eq!(data_fields, [DataValue::Null]);

        // Check value ignore
        let index_map = schema.build_index_map(&["id".to_string()]).unwrap();
        let mut temp = vec![DataValue::Scalar(ScalarValue::Text(
            "Will be ignored".to_string(),
        ))];
        schema.order_row(&index_map, &mut data_fields, &mut temp);
        assert_eq!(data_fields, [DataValue::Null]);
    }
}
