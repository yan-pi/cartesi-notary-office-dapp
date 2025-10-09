use crate::domain::{Document, NotarizationReceipt};
use crate::infrastructure::database::DocumentRepository;
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NotarizeError {
    #[error("Content cannot be empty")]
    EmptyContent,

    #[error("Filename cannot be empty")]
    EmptyFilename,

    #[error("Document with this content hash already exists")]
    DuplicateDocument,

    #[error("Database error: {0}")]
    DatabaseError(String),
}

pub struct NotarizeUseCase {
    repository: Box<dyn DocumentRepository>,
}

impl NotarizeUseCase {
    pub fn new(repository: Box<dyn DocumentRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(
        &self,
        content: &[u8],
        file_name: &str,
        mime_type: &str,
        submitted_by: &str,
        block_number: u64,
    ) -> Result<NotarizationReceipt, Box<dyn Error>> {
        // Validate inputs
        if content.is_empty() {
            return Err(Box::new(NotarizeError::EmptyContent));
        }

        if file_name.trim().is_empty() {
            return Err(Box::new(NotarizeError::EmptyFilename));
        }

        // Create document entity (generates hash and ID)
        let document = Document::new(content, file_name, mime_type, submitted_by);

        // Check for duplicate hash
        if self.repository.find_by_hash(&document.content_hash).is_ok() {
            return Err(Box::new(NotarizeError::DuplicateDocument));
        }

        // Save document to repository
        self.repository
            .save_document(&document)
            .map_err(|e| Box::new(NotarizeError::DatabaseError(e.to_string())) as Box<dyn Error>)?;

        // Generate notarization receipt
        let receipt = NotarizationReceipt::new(
            document.id.clone(),
            document.content_hash.clone(),
            document.created_at,
            block_number,
        );

        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::database::SqliteRepository;

    #[test]
    fn test_notarize_usecase_creation() {
        let repo = SqliteRepository::new_in_memory().unwrap();
        let _usecase = NotarizeUseCase::new(Box::new(repo));
    }
}
