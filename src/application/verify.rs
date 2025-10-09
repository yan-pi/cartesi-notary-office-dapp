use crate::domain::{Document, NotarizationReceipt};
use crate::infrastructure::database::DocumentRepository;
use serde::{Deserialize, Serialize};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VerifyError {
    #[error("Invalid hash format: must be 64 hexadecimal characters")]
    InvalidHashFormat,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub exists: bool,
    pub document: Option<Document>,
    pub receipt: Option<NotarizationReceipt>,
}

impl VerificationResult {
    pub fn not_found() -> Self {
        Self {
            exists: false,
            document: None,
            receipt: None,
        }
    }

    pub fn found(document: Document) -> Self {
        // Reconstruct receipt from document
        // Note: We don't have block_number stored in document yet
        // For MVP, we'll use 0 as placeholder or extend Document later
        let receipt = NotarizationReceipt::new(
            document.id.clone(),
            document.content_hash.clone(),
            document.created_at,
            0, // Placeholder - we'd need to store this or retrieve it differently
        );

        Self {
            exists: true,
            document: Some(document),
            receipt: Some(receipt),
        }
    }
}

pub struct VerifyUseCase {
    repository: Box<dyn DocumentRepository>,
}

impl VerifyUseCase {
    pub fn new(repository: Box<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, content_hash: &str) -> Result<VerificationResult, Box<dyn Error>> {
        // Validate hash format
        if !Self::is_valid_hash(content_hash) {
            return Err(Box::new(VerifyError::InvalidHashFormat));
        }

        // Query repository
        match self.repository.find_by_hash(content_hash) {
            Ok(document) => Ok(VerificationResult::found(document)),
            Err(_) => Ok(VerificationResult::not_found()),
        }
    }

    fn is_valid_hash(hash: &str) -> bool {
        // SHA-256 produces 64 hex characters
        hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::SqliteRepository;

    #[test]
    fn test_verify_usecase_creation() {
        let repo = SqliteRepository::new_in_memory().unwrap();
        let _usecase = VerifyUseCase::new(Box::new(repo));
    }

    #[test]
    fn test_is_valid_hash() {
        assert!(VerifyUseCase::is_valid_hash(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        ));
        assert!(VerifyUseCase::is_valid_hash(
            "ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"
        ));

        assert!(!VerifyUseCase::is_valid_hash("short"));
        assert!(!VerifyUseCase::is_valid_hash(
            "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"
        ));
        assert!(!VerifyUseCase::is_valid_hash(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefEXTRA"
        ));
    }
}
