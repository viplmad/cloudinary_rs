use chrono::{DateTime, Utc};
use serde::{de, Deserialize, Deserializer};
use std::{fmt::Display, str::FromStr};

fn deserialize_from_str<'de, S, D>(deserializer: D) -> Result<S, D::Error>
where
    S: FromStr,      // Required for S::from_str...
    S::Err: Display, // Required for .map_err(de::Error::custom)
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    S::from_str(&s).map_err(de::Error::custom)
}

#[derive(Clone, Deserialize, Debug)]
pub struct Error {
    pub error: Message,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Message {
    pub message: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct UploadResponse {
    pub asset_id: String,
    pub public_id: String,
    pub version: usize,
    pub version_id: String,
    pub signature: String,
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub resource_type: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub bytes: usize,
    pub r#type: String,
    pub etag: String,
    pub placeholder: bool,
    pub url: String,
    pub secure_url: String,
    pub original_filename: String,
    // Not in documentation but needed
    pub folder: String,
    pub overwritten: Option<bool>,
    pub api_key: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct RenameResponse {
    pub asset_id: String,
    pub public_id: String,
    pub version: usize,
    pub version_id: String,
    pub signature: String,
    pub width: usize,
    pub height: usize,
    pub format: String,
    pub resource_type: String,
    #[serde(deserialize_with = "deserialize_from_str")]
    pub created_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub bytes: usize,
    pub r#type: String,
    pub placeholder: bool,
    pub url: String,
    pub secure_url: String,
    // Not in documentation but needed
    pub folder: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct DeleteResponse {
    pub result: String,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum CloudinaryUploadResult {
    Succes(Box<UploadResponse>),
    Error(Box<Error>),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum CloudinaryRenameResult {
    Succes(Box<RenameResponse>),
    Error(Box<Error>),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum CloudinaryDeleteResult {
    Succes(Box<DeleteResponse>),
    Error(Box<Error>),
}
