use json::JsonValue;

/// Database helper for integration tests
/// Sets up a temporary database and cleans up on drop
pub struct TestDatabase {
    path: String,
}

impl TestDatabase {
    pub fn new() -> Self {
        let path = format!("/tmp/notary_test_{}.db", uuid::Uuid::new_v4());
        std::env::set_var("NOTARY_DB_PATH", &path);
        Self { path }
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        std::env::remove_var("NOTARY_DB_PATH");
        let _ = std::fs::remove_file(&self.path);
    }
}

/// Create a test advance_state request
pub fn create_advance_request(payload_json: &str, msg_sender: &str, block_number: u64) -> JsonValue {
    let payload_hex = hex::encode(payload_json);

    json::object! {
        "request_type" => "advance_state",
        "data" => json::object! {
            "payload" => payload_hex,
            "metadata" => json::object! {
                "msg_sender" => msg_sender,
                "block_number" => block_number,
                "timestamp" => 1234567890,
                "epoch_index" => 0,
                "input_index" => 0
            }
        }
    }
}

/// Create a test inspect_state request
pub fn create_inspect_request(payload_json: &str) -> JsonValue {
    let payload_hex = hex::encode(payload_json);

    json::object! {
        "request_type" => "inspect_state",
        "data" => json::object! {
            "payload" => payload_hex
        }
    }
}

/// Create a notarize action payload
pub fn create_notarize_payload(content: &[u8], file_name: &str, mime_type: &str) -> String {
    use base64::Engine;
    let content_base64 = base64::engine::general_purpose::STANDARD.encode(content);

    format!(
        r#"{{"action":"notarize","data":{{"content":"{}","file_name":"{}","mime_type":"{}"}}}}"#,
        content_base64, file_name, mime_type
    )
}

/// Create a verify payload for inspect requests (VerifyRequest format)
pub fn create_verify_payload(content_hash: &str) -> String {
    format!(
        r#"{{"content_hash":"{}"}}"#,
        content_hash
    )
}

/// Decode a hex-encoded payload
#[allow(dead_code)]
pub fn decode_hex_payload(hex_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex_str)?;
    Ok(std::str::from_utf8(&bytes)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_advance_request() {
        let payload = r#"{"test":"data"}"#;
        let req = create_advance_request(payload, "0x123", 100);

        assert_eq!(req["request_type"].as_str().unwrap(), "advance_state");
        assert_eq!(req["data"]["metadata"]["msg_sender"].as_str().unwrap(), "0x123");
        assert_eq!(req["data"]["metadata"]["block_number"].as_u64().unwrap(), 100);

        // Verify payload is hex-encoded
        let payload_hex = req["data"]["payload"].as_str().unwrap();
        let decoded = decode_hex_payload(payload_hex).unwrap();
        assert_eq!(decoded, payload);
    }

    #[test]
    fn test_create_notarize_payload() {
        let payload = create_notarize_payload(b"Hello", "test.txt", "text/plain");

        assert!(payload.contains("notarize"));
        assert!(payload.contains("test.txt"));
        assert!(payload.contains("text/plain"));

        // Verify it's valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["action"], "notarize");
    }

    #[test]
    fn test_create_verify_payload() {
        let hash = "a".repeat(64);
        let payload = create_verify_payload(&hash);

        assert!(payload.contains(&hash));

        // Verify it's valid JSON with correct format (VerifyRequest)
        let parsed: serde_json::Value = serde_json::from_str(&payload).unwrap();
        assert_eq!(parsed["content_hash"], hash);
    }
}
