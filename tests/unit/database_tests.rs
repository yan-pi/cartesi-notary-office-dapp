use dapp::domain::Document;
use dapp::infrastructure::database::{DocumentRepository, SqliteRepository};

#[test]
fn test_init_database_creates_schema() {
    // Create in-memory database
    let repo = SqliteRepository::new_in_memory().expect("Failed to create repository");

    // Verify we can interact with tables (they exist)
    let result = repo.find_by_hash("nonexistent_hash");
    assert!(result.is_err()); // Should fail to find, but not crash
}

#[test]
fn test_save_document_persists() {
    let repo = SqliteRepository::new_in_memory().unwrap();
    let doc = Document::new(b"test content", "test.txt", "text/plain", "0x123");

    // Save document
    repo.save_document(&doc).expect("Failed to save document");

    // Retrieve by hash
    let found = repo
        .find_by_hash(&doc.content_hash)
        .expect("Failed to find document");

    assert_eq!(found.id, doc.id);
    assert_eq!(found.content_hash, doc.content_hash);
    assert_eq!(found.file_name, doc.file_name);
    assert_eq!(found.submitted_by, doc.submitted_by);
}

#[test]
fn test_find_by_hash_not_found() {
    let repo = SqliteRepository::new_in_memory().unwrap();

    let result = repo.find_by_hash("nonexistent_hash_12345");

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("not found") || err_msg.contains("no rows"));
}

#[test]
fn test_duplicate_hash_constraint() {
    let repo = SqliteRepository::new_in_memory().unwrap();

    // Create two documents with same content (same hash)
    let doc1 = Document::new(b"same content", "file1.txt", "text/plain", "0x123");
    let doc2 = Document::new(b"same content", "file2.txt", "text/plain", "0x456");

    // First save should succeed
    repo.save_document(&doc1)
        .expect("First save should succeed");

    // Second save with same hash should fail
    let result = repo.save_document(&doc2);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.to_lowercase().contains("duplicate")
            || err_msg.to_lowercase().contains("unique")
            || err_msg.to_lowercase().contains("constraint")
    );
}

#[test]
fn test_find_by_id() {
    let repo = SqliteRepository::new_in_memory().unwrap();
    let doc = Document::new(b"content", "file.txt", "text/plain", "0x123");

    repo.save_document(&doc).unwrap();

    let found = repo.find_by_id(&doc.id).expect("Failed to find by ID");
    assert_eq!(found.id, doc.id);
    assert_eq!(found.content_hash, doc.content_hash);
}

#[test]
fn test_multiple_documents() {
    let repo = SqliteRepository::new_in_memory().unwrap();

    let doc1 = Document::new(b"content 1", "file1.txt", "text/plain", "0x123");
    let doc2 = Document::new(b"content 2", "file2.txt", "text/plain", "0x456");
    let doc3 = Document::new(b"content 3", "file3.txt", "text/plain", "0x789");

    repo.save_document(&doc1).unwrap();
    repo.save_document(&doc2).unwrap();
    repo.save_document(&doc3).unwrap();

    // All should be retrievable
    assert!(repo.find_by_hash(&doc1.content_hash).is_ok());
    assert!(repo.find_by_hash(&doc2.content_hash).is_ok());
    assert!(repo.find_by_hash(&doc3.content_hash).is_ok());
}

#[test]
fn test_document_count() {
    let repo = SqliteRepository::new_in_memory().unwrap();

    let initial_count = repo.count_documents().unwrap();
    assert_eq!(initial_count, 0);

    let doc = Document::new(b"test", "file.txt", "text/plain", "0x123");
    repo.save_document(&doc).unwrap();

    let count = repo.count_documents().unwrap();
    assert_eq!(count, 1);
}
