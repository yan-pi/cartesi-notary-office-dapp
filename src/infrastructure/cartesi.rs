use hyper::{Body, Client, Method, Request};
use std::error::Error;

/// Send a notice to the Cartesi Rollup HTTP server
///
/// Notices are verifiable outputs that can be proven on the base layer.
/// They should be used for important state changes like notarization receipts.
///
/// # Arguments
/// * `client` - Hyper HTTP client
/// * `server_url` - Base URL of the rollup server (e.g., "http://127.0.0.1:5004")
/// * `payload` - JSON string to send (will be hex-encoded)
pub async fn send_notice(
    client: &Client<hyper::client::HttpConnector>,
    server_url: &str,
    payload: &str,
) -> Result<(), Box<dyn Error>> {
    // Hex-encode the JSON payload
    let payload_hex = hex::encode(payload);

    // Build request body
    let body_json = json::object! {
        "payload" => payload_hex
    };

    // Send POST request to /notice endpoint
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/notice", server_url))
        .header("content-type", "application/json")
        .body(Body::from(body_json.dump()))?;

    let response = client.request(request).await?;

    // Check for success
    if !response.status().is_success() {
        return Err(format!("Failed to send notice: HTTP {}", response.status()).into());
    }

    println!("Notice sent successfully");
    Ok(())
}

/// Send a report to the Cartesi Rollup HTTP server
///
/// Reports are non-verifiable outputs used for logging and query results.
/// They should be used for inspect_state responses and diagnostics.
///
/// # Arguments
/// * `client` - Hyper HTTP client
/// * `server_url` - Base URL of the rollup server
/// * `payload` - JSON string to send (will be hex-encoded)
pub async fn send_report(
    client: &Client<hyper::client::HttpConnector>,
    server_url: &str,
    payload: &str,
) -> Result<(), Box<dyn Error>> {
    // Hex-encode the JSON payload
    let payload_hex = hex::encode(payload);

    // Build request body
    let body_json = json::object! {
        "payload" => payload_hex
    };

    // Send POST request to /report endpoint
    let request = Request::builder()
        .method(Method::POST)
        .uri(format!("{}/report", server_url))
        .header("content-type", "application/json")
        .body(Body::from(body_json.dump()))?;

    let response = client.request(request).await?;

    // Check for success
    if !response.status().is_success() {
        return Err(format!("Failed to send report: HTTP {}", response.status()).into());
    }

    println!("Report sent successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_encoding() {
        let json_payload = r#"{"test":"data"}"#;
        let hex_encoded = hex::encode(json_payload);

        // Verify it's valid hex
        assert!(hex_encoded.chars().all(|c| c.is_ascii_hexdigit()));

        // Verify we can decode it back
        let decoded = hex::decode(&hex_encoded).unwrap();
        let decoded_str = std::str::from_utf8(&decoded).unwrap();
        assert_eq!(decoded_str, json_payload);
    }
}
