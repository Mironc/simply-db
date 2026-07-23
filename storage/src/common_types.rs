#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Debug, Clone, PartialEq)]
pub enum DataValue {
    Null,
    Scalar(ScalarValue),
}
impl std::fmt::Display for DataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataValue::Null => write!(f, "NULL"),
            DataValue::Scalar(scalar_value) => write!(f, "{}", scalar_value),
        }
    }
}
/// Short way to create DataValue from ScalarValue
///
/// Example:
/// ```
/// # use storage::common_types::DataValue;
/// # use storage::scalar;
/// # use storage::common_types::ScalarValue;
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
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase")
)]
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

#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "camelCase"),
    serde(tag = "type", content = "payload")
)]
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

#[cfg(test)]
mod tests {
    use crate::common_types::{ScalarType, ScalarValue};

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
}
