use anyhow::{anyhow, Result};

use reqwest::{header::HeaderMap, header::HeaderValue, Client, StatusCode, Url};
struct CloudflareStreams {
    client: Client,
    base: Url,
    account_identifier: String,
}

impl CloudflareStreams {
    pub fn try_new(base_url: Url, account_identifier: String, api_key: &str) -> Result<Self> {
        let mut api_key_value = HeaderValue::from_str(api_key)?;
        api_key_value.set_sensitive(true);
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", api_key_value);
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/json;charset=UTF-8"),
        );
        Ok(Self {
            client: Client::builder().default_headers(headers).build()?,
            base: base_url,
            account_identifier,
        })
    }

    pub async fn get_tus_upload_url(&self, creator_name: &str, video_length: usize) -> Result<Url> {
        let endpoint = self
            .base
            .join("accounts")
            .and_then(|url| url.join(&self.account_identifier))
            .and_then(|url| url.join("stream"))?;
        let response = self
            .client
            .post(endpoint)
            .header("Tus-Resumable", "1.0.0")
            .header("Upload-Length", video_length)
            .header("Upload-Creator", creator_name)
            .send()
            .await?;
        let response = response.error_for_status()?;
        match (response.status(), response.headers().get("location")) {
            (StatusCode::CREATED, Some(location)) => {
                let location_str = location.to_str()?;
                let url = Url::parse(location_str)?;
                Ok(url)
            }
            _ => Err(anyhow!("Platform error")),
        }
    }
}
