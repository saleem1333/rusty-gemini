use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: Type,
    pub format: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub nullable: bool,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub max_items: Option<String>,
    pub min_items: Option<String>,
    pub properties: Option<HashMap<String, Box<Schema>>>,
    pub required: Option<Vec<String>>,
    pub items: Option<Box<Schema>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    TypeUnspecified,
    String,
    Number,
    Integer,
    Boolean,
    Array,
    Object,
}