//! JSON Standard Library
//!
//! JSON serialization and deserialization.

use serde_json::Value as JsonValue;

/// A simple JSON value representation for A16
#[derive(Debug, Clone)]
pub enum A16Json {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<A16Json>),
    Object(Vec<(String, A16Json)>),
}

/// Parse a JSON string into an A16Json value
pub fn parse(source: &str) -> Result<A16Json, String> {
    let json: JsonValue = serde_json::from_str(source)
        .map_err(|e| format!("JSON parse error: {}", e))?;
    Ok(from_serde(json))
}

/// Serialize an A16Json value to a JSON string
pub fn stringify(value: &A16Json) -> String {
    let json = to_serde(value);
    serde_json::to_string(&json).unwrap_or_else(|_| "null".to_string())
}

/// Pretty-print an A16Json value
pub fn stringify_pretty(value: &A16Json) -> String {
    let json = to_serde(value);
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "null".to_string())
}

/// Get a field from a JSON object by key
pub fn get_field<'a>(obj: &'a A16Json, key: &str) -> Option<&'a A16Json> {
    if let A16Json::Object(fields) = obj {
        fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    } else {
        None
    }
}

/// Get an element from a JSON array by index
pub fn get_index(arr: &A16Json, index: usize) -> Option<&A16Json> {
    if let A16Json::Array(items) = arr {
        items.get(index)
    } else {
        None
    }
}

/// Check if a value is null
pub fn is_null(value: &A16Json) -> bool {
    matches!(value, A16Json::Null)
}

/// Get string value
pub fn as_string(value: &A16Json) -> Option<&str> {
    if let A16Json::String(s) = value { Some(s) } else { None }
}

/// Get integer value
pub fn as_int(value: &A16Json) -> Option<i64> {
    if let A16Json::Int(n) = value { Some(*n) } else { None }
}

/// Get float value
pub fn as_float(value: &A16Json) -> Option<f64> {
    match value {
        A16Json::Float(n) => Some(*n),
        A16Json::Int(n) => Some(*n as f64),
        _ => None,
    }
}

/// Convert from serde_json::Value
fn from_serde(val: JsonValue) -> A16Json {
    match val {
        JsonValue::Null => A16Json::Null,
        JsonValue::Bool(b) => A16Json::Bool(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                A16Json::Int(i)
            } else {
                A16Json::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        JsonValue::String(s) => A16Json::String(s),
        JsonValue::Array(arr) => A16Json::Array(arr.into_iter().map(from_serde).collect()),
        JsonValue::Object(obj) => A16Json::Object(
            obj.into_iter().map(|(k, v)| (k, from_serde(v))).collect()
        ),
    }
}

/// Convert to serde_json::Value
fn to_serde(val: &A16Json) -> JsonValue {
    match val {
        A16Json::Null => JsonValue::Null,
        A16Json::Bool(b) => JsonValue::Bool(*b),
        A16Json::Int(n) => JsonValue::Number((*n).into()),
        A16Json::Float(n) => {
            serde_json::Number::from_f64(*n)
                .map(JsonValue::Number)
                .unwrap_or(JsonValue::Null)
        }
        A16Json::String(s) => JsonValue::String(s.clone()),
        A16Json::Array(arr) => JsonValue::Array(arr.iter().map(to_serde).collect()),
        A16Json::Object(obj) => {
            let map: serde_json::Map<String, JsonValue> = obj.iter()
                .map(|(k, v)| (k.clone(), to_serde(v)))
                .collect();
            JsonValue::Object(map)
        }
    }
}
