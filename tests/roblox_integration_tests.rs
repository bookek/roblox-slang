use roblox_slang::roblox::client::RobloxCloudClient;
use roblox_slang::roblox::{MergeStrategy, SyncOrchestrator};
use roblox_slang::Config;
use std::fs;

#[tokio::test]
async fn test_get_entries_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "entries": [
                {
                    "identifier": {
                        "key": "test.key",
                        "context": "",
                        "source": "Test"
                    },
                    "metadata": {
                        "example": null,
                        "entryType": "manual"
                    },
                    "translations": [
                        {
                            "locale": "en",
                            "translationText": "Test"
                        },
                        {
                            "locale": "es",
                            "translationText": "Prueba"
                        }
                    ]
                }
            ]
        }"#,
        )
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-table-id", None).await;

    match &result {
        Ok(entries) => println!("Success: got {} entries", entries.len()),
        Err(e) => println!("Error: {}", e),
    }

    assert!(result.is_ok());
    let entries = result.unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].identifier.key, "test.key");
    assert_eq!(entries[0].translations.len(), 2);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_entries_empty_table() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/empty-table/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"entries": []}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("empty-table", None).await;

    assert!(result.is_ok());
    let entries = result.unwrap();
    assert_eq!(entries.len(), 0);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_update_entries_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "PATCH",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id",
        )
        .match_header("x-api-key", "test_api_key")
        .match_header("content-type", "application/json")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"failedEntriesAndTranslations": [], "modifiedEntriesAndTranslations": []}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let entries = vec![];
    let result = client
        .update_table_entries("test-table-id", &entries, None)
        .await;

    assert!(result.is_ok());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_sync_uploads_entries_in_batches() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let input_dir = temp_dir.path().join("translations");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir(&input_dir).unwrap();

    let mut json = serde_json::Map::new();
    for index in 0..25 {
        json.insert(
            format!("key{}", index),
            serde_json::Value::String(format!("Value {}", index)),
        );
    }
    fs::write(
        input_dir.join("en.json"),
        serde_json::to_string(&serde_json::Value::Object(json)).unwrap(),
    )
    .unwrap();

    let mut server = mockito::Server::new_async().await;
    let get_mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/table-id/entries",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"entries":[]}"#)
        .create_async()
        .await;
    let patch_mock = server
        .mock(
            "PATCH",
            "/legacy-localization-tables/v1/localization-table/tables/table-id",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"failedEntriesAndTranslations":[],"modifiedEntriesAndTranslations":[]}"#)
        .expect(2)
        .create_async()
        .await;

    let config = Config {
        input_directory: input_dir.to_string_lossy().to_string(),
        output_directory: output_dir.to_string_lossy().to_string(),
        supported_locales: vec!["en".to_string()],
        base_locale: "en".to_string(),
        ..Default::default()
    };

    let mut client = RobloxCloudClient::new("test_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());
    let orchestrator = SyncOrchestrator::new(client, config);
    let stats = orchestrator
        .sync("table-id", MergeStrategy::Merge, false)
        .await
        .unwrap();

    assert_eq!(stats.entries_added, 25);
    get_mock.assert_async().await;
    patch_mock.assert_async().await;
}

#[tokio::test]
async fn test_get_table_metadata_success() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "id": "test-table-id",
            "name": "Test Table"
        }"#,
        )
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_metadata("test-table-id").await;

    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert_eq!(metadata.id, "test-table-id");
    assert_eq!(metadata.name, Some("Test Table".to_string()));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_authentication_error_401() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id/entries",
        )
        .match_header("x-api-key", "invalid_key")
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Unauthorized", "message": "Invalid API key"}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("invalid_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-table-id", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Invalid or expired API key"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_permission_error_403() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/forbidden-table/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Forbidden", "message": "Insufficient permissions"}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("forbidden-table", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Insufficient permissions"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_rate_limit_error_429() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(429)
        .with_header("content-type", "application/json")
        .with_header("Retry-After", "60")
        .with_body(r#"{"error": "TooManyRequests", "message": "Rate limit exceeded"}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-table-id", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Rate limit") || err_msg.contains("retry"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_server_error_500() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "InternalServerError", "message": "Server error"}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-table-id", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("server error") || err_msg.contains("Server error"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_server_error_503() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "PATCH",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(503)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{"error": "ServiceUnavailable", "message": "Service temporarily unavailable"}"#,
        )
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let entries = vec![];
    let result = client
        .update_table_entries("test-table-id", &entries, None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("server error") || err_msg.contains("Server error"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_invalid_table_id_400() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/invalid-id/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "BadRequest", "message": "Invalid table id"}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("invalid-id", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = err.to_string();
    assert!(err_msg.contains("Invalid table id") || err_msg.contains("400"));

    mock.assert_async().await;
}

#[tokio::test]
async fn test_network_timeout() {
    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing("http://127.0.0.1:9".to_string());
    let result = client.get_table_entries("test-table-id", None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_malformed_json_response() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-table-id/entries",
        )
        .match_header("x-api-key", "test_api_key")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"entries": [invalid json}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-table-id", None).await;

    assert!(result.is_err());

    mock.assert_async().await;
}

#[tokio::test]
async fn test_correct_endpoint_paths() {
    let mut server = mockito::Server::new_async().await;
    let mock_get = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-id/entries",
        )
        .match_header("x-api-key", "test_key")
        .with_status(200)
        .with_body(r#"{"entries": []}"#)
        .create_async()
        .await;
    let mock_patch = server
        .mock(
            "PATCH",
            "/legacy-localization-tables/v1/localization-table/tables/test-id",
        )
        .match_header("x-api-key", "test_key")
        .with_status(200)
        .with_body(r#"{"success": true}"#)
        .create_async()
        .await;
    let mock_metadata = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-id",
        )
        .match_header("x-api-key", "test_key")
        .with_status(200)
        .with_body(r#"{"id": "test-id", "name": "Test"}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());
    let _ = client.get_table_entries("test-id", None).await;
    let _ = client.update_table_entries("test-id", &[], None).await;
    let _ = client.get_table_metadata("test-id").await;
    mock_get.assert_async().await;
    mock_patch.assert_async().await;
    mock_metadata.assert_async().await;
}

#[tokio::test]
async fn test_api_key_header_present() {
    let mut server = mockito::Server::new_async().await;

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-id/entries",
        )
        .match_header("x-api-key", "my_secret_key_123")
        .with_status(200)
        .with_body(r#"{"entries": []}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("my_secret_key_123".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-id", None).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}

#[tokio::test]
async fn test_user_agent_header() {
    let mut server = mockito::Server::new_async().await;
    let expected_user_agent = format!("roblox-slang/{}", env!("CARGO_PKG_VERSION"));

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-id/entries",
        )
        .match_header("user-agent", expected_user_agent.as_str())
        .with_status(200)
        .with_body(r#"{"entries": []}"#)
        .create_async()
        .await;

    let mut client = RobloxCloudClient::new("test_key".to_string()).unwrap();
    client.set_base_url_for_testing(server.url());

    let result = client.get_table_entries("test-id", None).await;

    assert!(result.is_ok());
    mock.assert_async().await;
}
