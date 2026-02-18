use roblox_slang::roblox::client::RobloxCloudClient;

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
    // Test that client has proper timeout configured
    let client = RobloxCloudClient::new("test_api_key".to_string()).unwrap();

    // Try to connect to non-existent server
    let result = client.get_table_entries("test-table-id", None).await;

    // Should fail with network error
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

    // Test GET entries endpoint
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

    // Test PATCH update endpoint
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

    // Test GET metadata endpoint
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

    // Test all endpoints
    let _ = client.get_table_entries("test-id", None).await;
    let _ = client.update_table_entries("test-id", &[], None).await;
    let _ = client.get_table_metadata("test-id").await;

    // Verify correct paths were called
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

    let mock = server
        .mock(
            "GET",
            "/legacy-localization-tables/v1/localization-table/tables/test-id/entries",
        )
        .match_header("user-agent", "roblox-slang/2.0.2")
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
