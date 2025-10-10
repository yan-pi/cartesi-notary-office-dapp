use crate::domain::{Document, NotarizationReceipt};
use serde::{Deserialize, Serialize};

/// Request to notarize a document
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotarizeRequest {
    /// Base64-encoded document content
    pub content: String,
    /// Document filename
    pub file_name: String,
    /// MIME type (e.g., "application/pdf", "text/plain")
    pub mime_type: String,
}

/// Request to verify a document by hash
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VerifyRequest {
    /// SHA-256 hash (64 hex characters)
    pub content_hash: String,
}

/// Input action types that can be sent to the DApp
#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum InputAction {
    /// Notarize a new document (state-changing operation)
    Notarize { data: NotarizeRequest },

    /// Verify an existing document (can be query or state-changing)
    Verify { data: VerifyRequest },
}

/// Response sent as a Cartesi Notice (verifiable on-chain)
#[derive(Debug, Serialize)]
pub struct NoticeResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub receipt: NotarizationReceipt,
}

impl NoticeResponse {
    pub fn notarization(receipt: NotarizationReceipt) -> Self {
        Self {
            response_type: "notarization_receipt".to_string(),
            receipt,
        }
    }
}

/// Response sent as a Cartesi Report (not verifiable, for logs/queries)
#[derive(Debug, Serialize)]
pub struct ReportResponse {
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt: Option<NotarizationReceipt>,
}

impl ReportResponse {
    pub fn from_verification(result: &crate::application::VerificationResult) -> Self {
        Self {
            exists: result.exists,
            document: result.document.clone(),
            receipt: result.receipt.clone(),
        }
    }

    pub fn error(_message: &str) -> Self {
        // For error cases, we could extend this with an error field
        // For now, just return not found
        Self {
            exists: false,
            document: None,
            receipt: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_action_deserialize_notarize() {
        let json = r#"{"action":"notarize","data":{"content":"SGVsbG8=","file_name":"test.txt","mime_type":"text/plain"}}"#;
        let action: InputAction = serde_json::from_str(json).unwrap();

        match action {
            InputAction::Notarize { data } => {
                assert_eq!(data.file_name, "test.txt");
                assert_eq!(data.mime_type, "text/plain");
            }
            _ => panic!("Expected Notarize variant"),
        }
    }

    #[test]
    fn test_input_action_deserialize_verify() {
        let json = r#"{"action":"verify","data":{"content_hash":"abc123"}}"#;
        let action: InputAction = serde_json::from_str(json).unwrap();

        match action {
            InputAction::Verify { data } => {
                assert_eq!(data.content_hash, "abc123");
            }
            _ => panic!("Expected Verify variant"),
        }
    }

    #[test]
    fn test_notice_response_serialize() {
        use crate::domain::NotarizationReceipt;

        let receipt =
            NotarizationReceipt::new("doc-id".to_string(), "hash123".to_string(), 1234567890, 100);

        let response = NoticeResponse::notarization(receipt);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("notarization_receipt"));
        assert!(json.contains("doc-id"));
    }
}
