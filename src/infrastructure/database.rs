use crate::domain::Document;
use rusqlite::{params, Connection, OptionalExtension};
use std::error::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("Document not found")]
    NotFound,

    #[error("Duplicate document hash")]
    DuplicateHash,
}

pub trait DocumentRepository {
    fn save_document(&self, doc: &Document) -> Result<(), Box<dyn Error>>;
    fn find_by_hash(&self, hash: &str) -> Result<Document, Box<dyn Error>>;
    fn find_by_id(&self, id: &str) -> Result<Document, Box<dyn Error>>;
    fn count_documents(&self) -> Result<usize, Box<dyn Error>>;
}

pub struct SqliteRepository {
    conn: Connection,
}

impl SqliteRepository {
    pub fn new(path: &str) -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open(path)?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    pub fn new_in_memory() -> Result<Self, Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    fn init_schema(conn: &Connection) -> Result<(), Box<dyn Error>> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                content_hash TEXT UNIQUE NOT NULL,
                file_name TEXT NOT NULL,
                mime_type TEXT NOT NULL,
                submitted_by TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_content_hash ON documents(content_hash)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON documents(created_at)",
            [],
        )?;

        Ok(())
    }

    fn row_to_document(row: &rusqlite::Row) -> Result<Document, rusqlite::Error> {
        Ok(Document {
            id: row.get(0)?,
            content_hash: row.get(1)?,
            file_name: row.get(2)?,
            mime_type: row.get(3)?,
            submitted_by: row.get(4)?,
            created_at: row.get(5)?,
        })
    }
}

impl DocumentRepository for SqliteRepository {
    fn save_document(&self, doc: &Document) -> Result<(), Box<dyn Error>> {
        match self.conn.execute(
            "INSERT INTO documents (id, content_hash, file_name, mime_type, submitted_by, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &doc.id,
                &doc.content_hash,
                &doc.file_name,
                &doc.mime_type,
                &doc.submitted_by,
                &doc.created_at
            ],
        ) {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(err, _)) => {
                if err.code == rusqlite::ErrorCode::ConstraintViolation {
                    Err(Box::new(DatabaseError::DuplicateHash))
                } else {
                    Err(Box::new(rusqlite::Error::SqliteFailure(err, None)))
                }
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    fn find_by_hash(&self, hash: &str) -> Result<Document, Box<dyn Error>> {
        let doc = self
            .conn
            .query_row(
                "SELECT id, content_hash, file_name, mime_type, submitted_by, created_at
                 FROM documents
                 WHERE content_hash = ?1",
                params![hash],
                Self::row_to_document,
            )
            .optional()?;

        doc.ok_or_else(|| Box::new(DatabaseError::NotFound) as Box<dyn Error>)
    }

    fn find_by_id(&self, id: &str) -> Result<Document, Box<dyn Error>> {
        let doc = self
            .conn
            .query_row(
                "SELECT id, content_hash, file_name, mime_type, submitted_by, created_at
                 FROM documents
                 WHERE id = ?1",
                params![id],
                Self::row_to_document,
            )
            .optional()?;

        doc.ok_or_else(|| Box::new(DatabaseError::NotFound) as Box<dyn Error>)
    }

    fn count_documents(&self) -> Result<usize, Box<dyn Error>> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM documents", [], |row| row.get(0))?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_in_memory_db() {
        let repo = SqliteRepository::new_in_memory();
        assert!(repo.is_ok());
    }
}
