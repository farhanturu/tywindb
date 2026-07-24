#![allow(dead_code)]

use serde_json::Value as JsonValue;

use crate::types::Value;

/// Convert Tywindb Value to JSON Value
pub fn value_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Boolean(b) => JsonValue::Bool(*b),
        Value::Integer(i) => JsonValue::Number((*i).into()),
        Value::Float(f) => {
            if let Some(n) = serde_json::Number::from_f64(*f) {
                JsonValue::Number(n)
            } else {
                JsonValue::Null
            }
        }
        Value::Text(s) => JsonValue::String(s.clone()),
        Value::Blob(b) => JsonValue::String(format!("<blob {} bytes>", b.len())),
        Value::Array(arr) => JsonValue::Array(arr.iter().map(value_to_json).collect()),
        Value::Object(map) => {
            let obj: serde_json::Map<String, JsonValue> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_json(v)))
                .collect();
            JsonValue::Object(obj)
        }
    }
}

/// Convert JSON Value to Tywindb Value
pub fn json_to_value(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Boolean(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        JsonValue::String(s) => Value::Text(s.clone()),
        JsonValue::Array(arr) => Value::Array(arr.iter().map(json_to_value).collect()),
        JsonValue::Object(map) => {
            let obj: std::collections::HashMap<String, Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect();
            Value::Object(obj)
        }
    }
}

/// Parse JSON string to Tywindb Value
pub fn parse_json(json_str: &str) -> Result<Value, String> {
    let json: JsonValue = serde_json::from_str(json_str)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    Ok(json_to_value(&json))
}

/// Convert Tywindb Value to JSON string
pub fn to_json_string(value: &Value) -> Result<String, String> {
    let json = value_to_json(value);
    serde_json::to_string_pretty(&json)
        .map_err(|e| format!("Failed to serialize: {}", e))
}

/// Extract nested value from JSON-like structure
pub fn extract_nested(value: &Value, path: &[&str]) -> Option<Value> {
    if path.is_empty() {
        return Some(value.clone());
    }

    match value {
        Value::Object(map) => {
            let key = path[0];
            map.get(key).and_then(|v| extract_nested(v, &path[1..]))
        }
        Value::Array(arr) => {
            if let Ok(idx) = path[0].parse::<usize>() {
                arr.get(idx).and_then(|v| extract_nested(v, &path[1..]))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Set nested value in JSON-like structure
pub fn set_nested(value: &mut Value, path: &[&str], new_value: Value) -> bool {
    if path.is_empty() {
        return false;
    }

    if path.len() == 1 {
        match value {
            Value::Object(map) => {
                map.insert(path[0].to_string(), new_value);
                true
            }
            _ => false,
        }
    } else {
        match value {
            Value::Object(map) => {
                let key = path[0];
                if let Some(inner) = map.get_mut(key) {
                    set_nested(inner, &path[1..], new_value)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_conversion() {
        let value = Value::Object(
            vec![
                ("name".to_string(), Value::Text("John".to_string())),
                ("age".to_string(), Value::Integer(30)),
                ("active".to_string(), Value::Boolean(true)),
            ]
            .into_iter()
            .collect(),
        );

        let json = value_to_json(&value);
        assert_eq!(json["name"], "John");
        assert_eq!(json["age"], 30);
        assert_eq!(json["active"], true);

        let back = json_to_value(&json);
        assert_eq!(back, value);
    }

    #[test]
    fn test_parse_json() {
        let json_str = r#"{"name": "John", "age": 30}"#;
        let value = parse_json(json_str).unwrap();
        
        assert!(matches!(value, Value::Object(_)));
        if let Value::Object(map) = value {
            assert_eq!(map.get("name"), Some(&Value::Text("John".to_string())));
            assert_eq!(map.get("age"), Some(&Value::Integer(30)));
        }
    }

    #[test]
    fn test_extract_nested() {
        let value = Value::Object(
            vec![
                ("user".to_string(), Value::Object(
                    vec![
                        ("name".to_string(), Value::Text("John".to_string())),
                        ("address".to_string(), Value::Object(
                            vec![
                                ("city".to_string(), Value::Text("NYC".to_string())),
                            ]
                            .into_iter()
                            .collect(),
                        )),
                    ]
                    .into_iter()
                    .collect(),
                )),
            ]
            .into_iter()
            .collect(),
        );

        assert_eq!(
            extract_nested(&value, &["user", "name"]),
            Some(Value::Text("John".to_string()))
        );
        assert_eq!(
            extract_nested(&value, &["user", "address", "city"]),
            Some(Value::Text("NYC".to_string()))
        );
    }
}
