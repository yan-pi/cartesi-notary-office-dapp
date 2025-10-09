use dapp::domain::{Document, NotarizationReceipt};

#[cfg(test)]
mod document_tests {
    use super::*;

    #[test]
    fn test_document_generates_sha256_hash() {
        let content = b"test content";
        let doc = Document::new(content, "test.txt", "text/plain", "0x123");

        // SHA-256 produces 64 hex characters
        assert_eq!(doc.content_hash.len(), 64);
        assert!(doc.content_hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_document_hash_is_deterministic() {
        let content = b"same content";
        let doc1 = Document::new(content, "file1.txt", "text/plain", "0x123");
        let doc2 = Document::new(content, "file2.txt", "text/plain", "0x456");

        // Same content should produce same hash regardless of other fields
        assert_eq!(doc1.content_hash, doc2.content_hash);
    }

    #[test]
    fn test_document_different_content_different_hash() {
        let doc1 = Document::new(b"content one", "file.txt", "text/plain", "0x123");
        let doc2 = Document::new(b"content two", "file.txt", "text/plain", "0x123");

        // Different content should produce different hashes
        assert_ne!(doc1.content_hash, doc2.content_hash);
    }

    #[test]
    fn test_document_generates_unique_id() {
        let content = b"test";
        let doc1 = Document::new(content, "file.txt", "text/plain", "0x123");
        let doc2 = Document::new(content, "file.txt", "text/plain", "0x123");

        // Each document should get a unique UUID
        assert_ne!(doc1.id, doc2.id);
    }

    #[test]
    fn test_document_timestamp_is_set() {
        let doc = Document::new(b"test", "file.txt", "text/plain", "0x123");

        // Timestamp should be set to current time (reasonable range)
        let now = chrono::Utc::now().timestamp();
        assert!(doc.created_at > 0);
        assert!(doc.created_at <= now);
        assert!((now - doc.created_at) < 2); // Within 2 seconds
    }

    #[test]
    fn test_document_stores_metadata() {
        let doc = Document::new(b"test", "my_file.pdf", "application/pdf", "0xABCD");

        assert_eq!(doc.file_name, "my_file.pdf");
        assert_eq!(doc.mime_type, "application/pdf");
        assert_eq!(doc.submitted_by, "0xABCD");
    }
}

#[cfg(test)]
mod receipt_tests {
    use super::*;

    #[test]
    fn test_receipt_proof_format() {
        let receipt = NotarizationReceipt {
            document_id: "test-id".to_string(),
            content_hash: "abcd1234".to_string(),
            notarized_at: 1234567890,
            block_number: 12345,
            proof: format!("sha256:{}@{}", "abcd1234", 1234567890),
        };

        assert_eq!(receipt.proof, "sha256:abcd1234@1234567890");
    }

    #[test]
    fn test_receipt_contains_all_fields() {
        let receipt = NotarizationReceipt {
            document_id: "doc-123".to_string(),
            content_hash: "hash123".to_string(),
            notarized_at: 9999,
            block_number: 100,
            proof: "proof".to_string(),
        };

        assert_eq!(receipt.document_id, "doc-123");
        assert_eq!(receipt.content_hash, "hash123");
        assert_eq!(receipt.notarized_at, 9999);
        assert_eq!(receipt.block_number, 100);
        assert!(!receipt.proof.is_empty());
    }
}
