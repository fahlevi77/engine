// siddhi_rust/src/core/util/attribute_converter.rs
// Utility helpers for converting between arbitrary values and AttributeValue
// according to the desired Attribute::Type.  This is a light weight port of
// io.siddhi.core.util.AttributeConverter.

use crate::core::event::value::AttributeValue;
use crate::query_api::definition::attribute::Type as AttributeType;

/// Convert a generic AttributeValue to the requested `AttributeType`.
/// If the incoming value already matches the target type no conversion is done.
/// The conversion rules mirror the Java implementation where possible.
pub fn get_property_value(
    value: AttributeValue,
    attribute_type: AttributeType,
) -> Result<AttributeValue, String> {
    match attribute_type {
        AttributeType::BOOL => match value {
            AttributeValue::Bool(_) => Ok(value),
            AttributeValue::String(s) => s
                .parse::<bool>()
                .map(AttributeValue::Bool)
                .map_err(|e| e.to_string()),
            _ => Err(format!("Unsupported mapping to BOOL for value {:?}", value)),
        },
        AttributeType::DOUBLE => match value {
            AttributeValue::Double(_) => Ok(value),
            AttributeValue::Float(f) => Ok(AttributeValue::Double(f as f64)),
            AttributeValue::String(s) => s
                .parse::<f64>()
                .map(AttributeValue::Double)
                .map_err(|e| e.to_string()),
            _ => Err(format!(
                "Unsupported mapping to DOUBLE for value {:?}",
                value
            )),
        },
        AttributeType::FLOAT => match value {
            AttributeValue::Float(_) => Ok(value),
            AttributeValue::String(s) => s
                .parse::<f32>()
                .map(AttributeValue::Float)
                .map_err(|e| e.to_string()),
            _ => Err(format!(
                "Unsupported mapping to FLOAT for value {:?}",
                value
            )),
        },
        AttributeType::INT => match value {
            AttributeValue::Int(_) => Ok(value),
            AttributeValue::String(s) => s
                .parse::<i32>()
                .map(AttributeValue::Int)
                .map_err(|e| e.to_string()),
            _ => Err(format!("Unsupported mapping to INT for value {:?}", value)),
        },
        AttributeType::LONG => match value {
            AttributeValue::Long(_) => Ok(value),
            AttributeValue::Int(i) => Ok(AttributeValue::Long(i as i64)),
            AttributeValue::String(s) => s
                .parse::<i64>()
                .map(AttributeValue::Long)
                .map_err(|e| e.to_string()),
            _ => Err(format!("Unsupported mapping to LONG for value {:?}", value)),
        },
        AttributeType::STRING => Ok(AttributeValue::String(match value {
            AttributeValue::String(s) => s,
            AttributeValue::Int(i) => i.to_string(),
            AttributeValue::Long(l) => l.to_string(),
            AttributeValue::Float(f) => f.to_string(),
            AttributeValue::Double(d) => d.to_string(),
            AttributeValue::Bool(b) => b.to_string(),
            AttributeValue::Null => "null".to_string(),
            AttributeValue::Object(_) => {
                return Err("Cannot automatically convert OBJECT to STRING".to_string())
            }
        })),
        AttributeType::OBJECT => Ok(value),
    }
}

/// Parse a string and convert it directly to an `AttributeValue` of the given type.
pub fn get_property_value_from_str(
    text: &str,
    attribute_type: AttributeType,
) -> Result<AttributeValue, String> {
    let value = AttributeValue::String(text.to_string());
    get_property_value(value, attribute_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_string_to_int() {
        let v = get_property_value_from_str("42", AttributeType::INT).unwrap();
        assert!(matches!(v, AttributeValue::Int(42)));
    }

    #[test]
    fn test_convert_int_to_long() {
        let v = get_property_value(AttributeValue::Int(7), AttributeType::LONG).unwrap();
        assert!(matches!(v, AttributeValue::Long(7)));
    }
}
