// This module exposes the handlers for integration testing
// In production, these are only used from main.rs

use crate::application::{
    InputAction, NotarizeUseCase, NoticeResponse, ReportResponse, VerifyUseCase,
};
use crate::infrastructure::{
    cartesi::{send_notice, send_report},
    database::{DocumentRepository, SqliteRepository},
};
use json::JsonValue;

// Database path - use persistent DB in production, in-memory for fallback
const DB_PATH: &str = "/var/lib/notary/notary.db";

/// Get a repository instance
/// In production, uses persistent SQLite database
/// Can be overridden via NOTARY_DB_PATH environment variable (for testing)
/// Falls back to in-memory if persistent fails
pub fn get_repository() -> Box<dyn DocumentRepository> {
    let db_path = std::env::var("NOTARY_DB_PATH").unwrap_or_else(|_| DB_PATH.to_string());
    Box::new(
        SqliteRepository::new(&db_path)
            .or_else(|_| SqliteRepository::new_in_memory())
            .expect("Failed to initialize database"),
    )
}

pub async fn handle_advance(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Received advance request");

    // Extract hex-encoded payload
    let payload_hex = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;

    // Decode from hex to bytes
    let payload_bytes = hex::decode(payload_hex)?;
    let payload_str = std::str::from_utf8(&payload_bytes)?;

    println!("Decoded payload: {}", payload_str);

    // Parse input action
    let input: InputAction = match serde_json::from_str(payload_str) {
        Ok(action) => action,
        Err(e) => {
            eprintln!("Failed to parse input action: {}", e);
            let error_msg = format!("{{\"error\":\"Invalid input format: {}\"}}", e);
            send_report(client, server_addr, &error_msg).await?;
            return Ok("reject");
        }
    };

    // Extract metadata
    let submitter = request["data"]["metadata"]["msg_sender"]
        .as_str()
        .unwrap_or("0x0000000000000000000000000000000000000000");

    let block_number = request["data"]["metadata"]["block_number"]
        .as_u64()
        .unwrap_or(0);

    // Handle different actions
    match input {
        InputAction::Notarize { data } => {
            println!(
                "Notarizing document: {} ({})",
                data.file_name, data.mime_type
            );

            // Decode base64 content
            use base64::Engine;
            let content = match base64::engine::general_purpose::STANDARD.decode(&data.content) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to decode base64 content: {}", e);
                    let error_msg = format!("{{\"error\":\"Invalid base64 content: {}\"}}", e);
                    send_report(client, server_addr, &error_msg).await?;
                    return Ok("reject");
                }
            };

            // Create use case with repository
            let notarize_usecase = NotarizeUseCase::new(get_repository());

            // Execute notarization
            match notarize_usecase.execute(
                &content,
                &data.file_name,
                &data.mime_type,
                submitter,
                block_number,
            ) {
                Ok(receipt) => {
                    println!("Document notarized successfully: {}", receipt.document_id);

                    // Send notice with receipt
                    let response = NoticeResponse::notarization(receipt);
                    let notice_json = serde_json::to_string(&response)?;
                    send_notice(client, server_addr, &notice_json).await?;

                    Ok("accept")
                }
                Err(e) => {
                    eprintln!("Notarization failed: {}", e);
                    let error_msg = format!("{{\"error\":\"{}\"}}", e);
                    send_report(client, server_addr, &error_msg).await?;
                    Ok("reject")
                }
            }
        }
        InputAction::Verify { data } => {
            println!("Verifying document hash: {}", data.content_hash);

            // Create use case
            let verify_usecase = VerifyUseCase::new(get_repository());

            // Execute verification
            match verify_usecase.execute(&data.content_hash) {
                Ok(result) => {
                    println!(
                        "Verification result: {}",
                        if result.exists { "found" } else { "not found" }
                    );

                    // Send report with result
                    let response = ReportResponse::from_verification(&result);
                    let report_json = serde_json::to_string(&response)?;
                    send_report(client, server_addr, &report_json).await?;

                    Ok("accept")
                }
                Err(e) => {
                    eprintln!("Verification failed: {}", e);
                    let error_msg = format!("{{\"error\":\"{}\"}}", e);
                    send_report(client, server_addr, &error_msg).await?;
                    Ok("reject")
                }
            }
        }
    }
}

pub async fn handle_inspect(
    client: &hyper::Client<hyper::client::HttpConnector>,
    server_addr: &str,
    request: JsonValue,
) -> Result<&'static str, Box<dyn std::error::Error>> {
    println!("Received inspect request");

    // Extract hex-encoded payload
    let payload_hex = request["data"]["payload"]
        .as_str()
        .ok_or("Missing payload")?;

    // Decode from hex to bytes
    let payload_bytes = hex::decode(payload_hex)?;
    let payload_str = std::str::from_utf8(&payload_bytes)?;

    println!("Decoded payload: {}", payload_str);

    // Parse verify request
    let verify_req: crate::application::VerifyRequest = match serde_json::from_str(payload_str) {
        Ok(req) => req,
        Err(e) => {
            eprintln!("Failed to parse verify request: {}", e);
            let error_msg = format!("{{\"error\":\"Invalid request format: {}\"}}", e);
            send_report(client, server_addr, &error_msg).await?;
            return Ok("accept"); // Inspect always accepts, errors go in reports
        }
    };

    println!("Verifying hash: {}", verify_req.content_hash);

    // Create use case
    let verify_usecase = VerifyUseCase::new(get_repository());

    // Execute verification
    match verify_usecase.execute(&verify_req.content_hash) {
        Ok(result) => {
            println!(
                "Verification result: {}",
                if result.exists { "found" } else { "not found" }
            );

            // Send report with result
            let response = ReportResponse::from_verification(&result);
            let report_json = serde_json::to_string(&response)?;
            send_report(client, server_addr, &report_json).await?;

            Ok("accept")
        }
        Err(e) => {
            eprintln!("Verification failed: {}", e);
            let error_msg = format!("{{\"error\":\"{}\"}}", e);
            send_report(client, server_addr, &error_msg).await?;
            Ok("accept") // Inspect always accepts
        }
    }
}
