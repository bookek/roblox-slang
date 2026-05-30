use crate::roblox::types::{CloudSyncError, GetTableEntriesResponse, LocalizationEntry};
use anyhow::{Context, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Deserialize;
use std::time::Duration;
pub struct RobloxCloudClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
}
#[derive(Debug, Deserialize)]
pub struct UpdateResponse {
    #[serde(rename = "failedEntriesAndTranslations")]
    #[allow(dead_code)]
    pub failed_entries: Vec<serde_json::Value>,
    #[serde(rename = "modifiedEntriesAndTranslations")]
    #[allow(dead_code)]
    pub modified_entries: Vec<serde_json::Value>,
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TableMetadata {
    pub id: String,
    pub name: Option<String>,
}

impl RobloxCloudClient {
    pub fn new(api_key: String) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("roblox-slang/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_key,
            base_url: "https://apis.roblox.com".to_string(),
        })
    }
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn set_base_url_for_testing(&mut self, url: String) {
        self.base_url = url;
    }
    pub async fn get_table_entries(
        &self,
        table_id: &str,
        game_id: Option<&str>,
    ) -> Result<Vec<LocalizationEntry>> {
        let mut url = format!(
            "{}/legacy-localization-tables/v1/localization-table/tables/{}/entries",
            self.base_url, table_id
        );

        if let Some(gid) = game_id {
            url.push_str(&format!("?gameId={}", gid));
        }

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to send GET request")?;
        if !response.status().is_success() {
            return self.handle_error_response(response).await;
        }
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;
        if let Ok(response_data) = serde_json::from_str::<GetTableEntriesResponse>(&response_text) {
            return Ok(response_data.entries);
        }
        if let Ok(entries) = serde_json::from_str::<Vec<LocalizationEntry>>(&response_text) {
            return Ok(entries);
        }
        anyhow::bail!("Failed to parse response. Body: {}", response_text);
    }
    pub async fn update_table_entries(
        &self,
        table_id: &str,
        entries: &[LocalizationEntry],
        game_id: Option<&str>,
    ) -> Result<UpdateResponse> {
        let mut url = format!(
            "{}/legacy-localization-tables/v1/localization-table/tables/{}",
            self.base_url, table_id
        );

        if let Some(gid) = game_id {
            url.push_str(&format!("?gameId={}", gid));
        }
        let request_body = serde_json::json!({
            "entries": entries
        });

        let response = self
            .client
            .patch(&url)
            .header("x-api-key", &self.api_key)
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send PATCH request")?;
        if !response.status().is_success() {
            return self.handle_error_response(response).await;
        }

        let update_response: UpdateResponse = response
            .json()
            .await
            .context("Failed to parse response JSON")?;

        Ok(update_response)
    }
    #[allow(dead_code)]
    pub async fn get_table_metadata(&self, table_id: &str) -> Result<TableMetadata> {
        let url = format!(
            "{}/legacy-localization-tables/v1/localization-table/tables/{}",
            self.base_url, table_id
        );

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to send GET request")?;
        if !response.status().is_success() {
            return self.handle_error_response(response).await;
        }

        let metadata: TableMetadata = response
            .json()
            .await
            .context("Failed to parse response JSON")?;

        Ok(metadata)
    }
    #[allow(dead_code)]
    pub async fn list_tables(
        &self,
        universe_id: &str,
    ) -> Result<Vec<crate::roblox::types::TableInfo>> {
        let url = format!(
            "{}/cloud/v2/universes/{}/localization-tables",
            self.base_url, universe_id
        );

        let response = self
            .client
            .get(&url)
            .header("x-api-key", &self.api_key)
            .send()
            .await
            .context("Failed to send GET request")?;
        if !response.status().is_success() {
            return self.handle_error_response(response).await;
        }

        let list_response: crate::roblox::types::ListTablesResponse = response
            .json()
            .await
            .context("Failed to parse response JSON")?;

        Ok(list_response.data)
    }
    #[allow(dead_code)]
    pub async fn resolve_table_id(&self, id: &str) -> Result<String> {
        if id.contains('-') && id.len() == 36 {
            return Ok(id.to_string());
        }
        if id.parse::<u64>().is_ok() {
            let tables = self
                .list_tables(id)
                .await
                .context("Failed to list tables for universe")?;

            if tables.is_empty() {
                anyhow::bail!("No localization tables found for universe {}", id);
            }
            return Ok(tables[0].id.clone());
        }
        anyhow::bail!("Invalid table ID or universe ID format: {}", id);
    }
    async fn handle_error_response<T>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();
        let status_code = status.as_u16();
        let retry_after_header = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());

        match status_code {
            401 => Err(CloudSyncError::AuthenticationError(format!(
                "Invalid or expired API key. Please check your credentials.\n\
                 \n\
                 Response: {}",
                error_body
            ))
            .into()),
            403 => Err(CloudSyncError::AuthenticationError(format!(
                "Insufficient permissions for this operation.\n\
                 \n\
                 Make sure your API key has access to this table.\n\
                 Response: {}",
                error_body
            ))
            .into()),
            429 => {
                let retry_after = retry_after_header.unwrap_or(1);

                Err(CloudSyncError::RateLimitError {
                    retry_after,
                    attempt: 1, // Placeholder, the limiter loop tracks attempts
                }
                .into())
            }
            500..=599 => Err(CloudSyncError::ServerError {
                status: status_code,
                message: format!(
                    "Roblox server error. Please try again later.\n\
                     \n\
                     Response: {}",
                    error_body
                ),
            }
            .into()),
            _ => Err(CloudSyncError::ApiError(format!(
                "API request failed with status {}: {}",
                status_code, error_body
            ))
            .into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = RobloxCloudClient::new("test_api_key".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_has_correct_base_url() {
        let client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
        assert_eq!(client.base_url, "https://apis.roblox.com");
    }

    #[test]
    fn test_client_creation_with_empty_key() {
        let client = RobloxCloudClient::new("".to_string());
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_stores_api_key() {
        let api_key = "test_key_12345".to_string();
        let client = RobloxCloudClient::new(api_key.clone()).unwrap();
        assert_eq!(client.api_key, api_key);
    }
}
