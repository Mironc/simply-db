use storage::{common_types::DataValue, schema::Schema};

#[derive(Debug, Clone, Default)]
pub struct Context<'a> {
    fields: Option<&'a Vec<DataValue>>,
    schema: Option<&'a Schema>,
}
impl<'a> Context<'a> {
    pub fn new(fields: &'a Vec<DataValue>, schema: &'a Schema) -> Self {
        Self {
            fields: Some(fields),
            schema: Some(schema),
        }
    }
    pub fn get_field(&self, field: &str) -> Option<&'a DataValue> {
        if let Some(fields) = self.fields
            && let Some(schema) = self.schema
        {
            schema
                .fields()
                .get_index(field)
                .map(|x| fields.get(x))
                .flatten()
        } else {
            None
        }
    }

    pub fn fields(&self) -> Option<&'a Vec<DataValue>> {
        self.fields
    }
}
