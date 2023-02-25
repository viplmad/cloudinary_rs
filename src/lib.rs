pub mod result;
pub mod upload;

use chrono::Utc;
use itertools::Itertools;
use reqwest::multipart::{Form, Part};
use reqwest::{Body, Client};
use sha1::{Digest, Sha1};
use std::collections::BTreeMap;
use std::fs::File;
use std::str::FromStr;
use tokio_util::codec::{BytesCodec, FramedRead};

use result::CloudinaryResult;
use upload::UploadOptions;

const API_BASE_URL: &str = "https://api.cloudinary.com/v1_1";

const UPLOAD_OPTION_API_KEY: &str = "api_key";
const UPLOAD_OPTION_TIMESTAMP: &str = "timestamp";
const UPLOAD_OPTION_RESOURCE_TYPE: &str = "resource_type";
const UPLOAD_OPTION_SIGNATURE: &str = "signature";

const QUERY_PARAM_SEPARATOR: &str = "&";

#[derive(Clone, Default)]
pub struct Cloudinary {
    pub cloud_name: String,
    api_key: i64,
    api_secret: String,
}

pub struct CloudinaryError(pub String);

impl Cloudinary {
    pub fn new(cloud_name: &str, api_key: i64, api_secret: &str) -> Self {
        Self {
            cloud_name: cloud_name.to_string(),
            api_key,
            api_secret: api_secret.to_string(),
        }
    }

    pub async fn upload_image(
        &self,
        src: File,
        filename: &str,
        options: &UploadOptions<'_>,
    ) -> Result<CloudinaryResult, CloudinaryError> {
        let file = prepare_file(src, filename).await?;
        let multipart = self
            .build_form_data(&mut options.get_map())
            .part("file", file);

        let response = Client::new()
            .post(format!("{}/{}/image/upload", API_BASE_URL, self.cloud_name))
            .multipart(multipart)
            .send()
            .await
            .map_err(|err| CloudinaryError(err.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|err| CloudinaryError(err.to_string()))?;
        serde_json::from_str(&text).map_err(|err| CloudinaryError(err.to_string()))
    }

    /// Renames an image
    /// ```rust
    /// use cloudinary::{Cloudinary};
    /// let cloudinary = Cloudinary::new("api_key".to_string(), "cloud_name".to_string(), "api_secret".to_string() );
    /// let result = cloudinary.rename_image("file.jpg", "new_file.jpg");
    /// ```
    pub async fn rename_image(
        &self,
        public_id: &str,
        new_public_id: &str,
    ) -> Result<CloudinaryResult, CloudinaryError> {
        let mut options_map = BTreeMap::<String, String>::new();
        options_map.insert("from_public_id".to_string(), public_id.to_string());
        options_map.insert("to_public_id".to_string(), new_public_id.to_string());

        let multipart = self.build_form_data(&mut options_map);

        let response = Client::new()
            .post(format!("{}/{}/image/rename", API_BASE_URL, self.cloud_name))
            .multipart(multipart)
            .send()
            .await
            .map_err(|err| CloudinaryError(err.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|err| CloudinaryError(err.to_string()))?;
        serde_json::from_str(&text).map_err(|err| CloudinaryError(err.to_string()))
    }

    /// Deletes an image
    /// ```rust
    /// use cloudinary::{Cloudinary};
    /// let cloudinary = Cloudinary::new("api_key".to_string(), "cloud_name".to_string(), "api_secret".to_string() );
    /// let result = cloudinary.delete_image("file.jpg");
    /// ```
    pub async fn delete_image(&self, public_id: &str) -> Result<CloudinaryResult, CloudinaryError> {
        let mut options_map = BTreeMap::<String, String>::new();
        options_map.insert("public_id".to_string(), public_id.to_string());

        let multipart = self.build_form_data(&mut options_map);

        let response = Client::new()
            .post(format!(
                "{}/{}/image/destroy",
                API_BASE_URL, self.cloud_name
            ))
            .multipart(multipart)
            .send()
            .await
            .map_err(|err| CloudinaryError(err.to_string()))?;

        let text = response
            .text()
            .await
            .map_err(|err| CloudinaryError(err.to_string()))?;
        serde_json::from_str(&text).map_err(|err| CloudinaryError(err.to_string()))
    }

    fn build_form_data(&self, options_map: &mut BTreeMap<String, String>) -> Form {
        let timestamp = Utc::now().timestamp_millis().to_string();

        let mut form = Form::new()
            .text(UPLOAD_OPTION_API_KEY, self.api_key.to_string())
            .text(UPLOAD_OPTION_TIMESTAMP, timestamp.clone());

        if let Some(resource_type) = options_map.remove(UPLOAD_OPTION_RESOURCE_TYPE) {
            form = form.text(UPLOAD_OPTION_RESOURCE_TYPE, resource_type);
        }

        let signature = self.build_signature(options_map, timestamp);

        form = form.text(UPLOAD_OPTION_SIGNATURE, signature);
        for (k, v) in options_map.iter() {
            form = form.text(k.clone(), v.clone());
        }
        form
    }

    fn build_signature(&self, map: &BTreeMap<String, String>, timestamp: String) -> String {
        let mut hasher = Sha1::new();
        if !map.is_empty() {
            let options_string = map
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .join(QUERY_PARAM_SEPARATOR);
            hasher.update(options_string);
            hasher.update(QUERY_PARAM_SEPARATOR);
        }
        hasher.update(format!("{}={}", UPLOAD_OPTION_TIMESTAMP, timestamp));
        hasher.update(&self.api_secret);

        format!("{:x}", hasher.finalize())
    }
}

/// Create connection options from URI cloudinary://<apiKey>:<apiSecret>@<cloudName>
impl FromStr for Cloudinary {
    type Err = CloudinaryError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: url::Url = s
            .parse()
            .map_err(|_| CloudinaryError(String::from("Url cannot be parsed")))?;

        let cloud_name = if let Some(cloud_name) = url.host_str() {
            Ok(cloud_name)
        } else {
            Err(CloudinaryError(String::from("Missing cloud name.")))
        }?;

        let api_key_string = url.username();
        let api_key = if !api_key_string.is_empty() {
            Ok(api_key_string
                .parse()
                .map_err(|_| CloudinaryError(String::from("Api key is not a number.")))?)
        } else {
            Err(CloudinaryError(String::from("Missing api key.")))
        }?;

        let api_secret = if let Some(api_secret) = url.password() {
            Ok(api_secret)
        } else {
            Err(CloudinaryError(String::from("Missing api secret.")))
        }?;

        Ok(Cloudinary::new(cloud_name, api_key, api_secret))
    }
}

async fn prepare_file(file: File, filename: &str) -> Result<Part, CloudinaryError> {
    let stream = FramedRead::new(tokio::fs::File::from_std(file), BytesCodec::new());
    let file_body = Body::wrap_stream(stream);
    Part::stream(file_body)
        .file_name(filename.to_string())
        .mime_str("image/*")
        .map_err(|err| CloudinaryError(err.to_string()))
}
