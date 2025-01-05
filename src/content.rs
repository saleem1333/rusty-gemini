use base64::{engine::general_purpose, Engine};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Content {
    pub role: Role,
    pub parts: Vec<Part>,
}

impl Content {
    pub fn user(value: impl Into<Part>) -> Self {
        Content {
            role: Role::User,
            parts: vec![value.into()],
        }
    }

    pub fn model(value: impl Into<Part>) -> Self {
        Content {
            role: Role::Model,
            parts: vec![value.into()],
        }
    }
}

impl<T> From<T> for Content where T: Into<Part> {
    fn from(value: T) -> Self {
        Content::user(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Part {
    Text(String),
    #[serde(rename = "inlineData")]
    Data {
        #[serde(serialize_with = "ser_data")]
        #[serde(deserialize_with = "des_data")]
        data: Vec<u8>,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

fn ser_data<S>(bytes: &Vec<u8>, ser: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    ser.serialize_str(&general_purpose::STANDARD.encode(bytes))
}

fn des_data<'de, D>(des: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(general_purpose::STANDARD.decode(String::deserialize(des)?).unwrap())
}
impl From<&str> for Part {
    fn from(value: &str) -> Self {
        Part::Text(value.to_string())
    }
}

impl From<String> for Part {
    fn from(value: String) -> Self {
        Part::Text(value)
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Model,
}
