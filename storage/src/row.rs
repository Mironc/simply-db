use crate::common_types::DataValue;

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    data: Vec<DataValue>,
}

impl Row {
    pub fn new(data: Vec<DataValue>) -> Self {
        Self { data }
    }

    pub fn data(&self) -> &Vec<DataValue> {
        &self.data
    }
    pub fn data_mut(&mut self) -> &mut Vec<DataValue> {
        &mut self.data
    }
}
