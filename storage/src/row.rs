use crate::common_types::SchemaValue;
#[derive(Debug, Clone)]
pub struct Row {
    data: SchemaValue,
}

impl Row {
    pub fn new(data: SchemaValue) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &SchemaValue {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut SchemaValue {
        &mut self.data
    }
}
