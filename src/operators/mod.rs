use serde_json::Value;
use std::collections::HashMap;

pub mod operators;
pub mod sinks;
pub mod sources;

pub use operators::*;
pub use sinks::*;
pub use sources::*;

pub static LAT_PORT: &str = "Data";
pub static THR_PORT: &str = "Data";

pub fn dict_merge(v: &Value, fields: &HashMap<String, Value>) -> Value {
    match v {
        Value::Object(m) => {
            let mut m = m.clone();
            for (k, v) in fields {
                m.insert(k.clone(), v.clone());
            }
            Value::Object(m)
        }
        v => v.clone(),
    }
}
