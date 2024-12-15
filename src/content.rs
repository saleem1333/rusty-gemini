use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: Role,
    pub parts: Vec<Part>,
}

impl Content {
    pub fn text(value: &str) -> Self {
        From::from(value)
    }

    pub fn model(value: &str) -> Self {
        Content {
            role: Role::Model,
            parts: vec![Part::Text(value.to_string())],
        }
    }
}

impl From<&str> for Content {
    fn from(value: &str) -> Self {
        Content {
            role: Role::User,
            parts: vec![Part::Text(value.to_string())],
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Model,
}
