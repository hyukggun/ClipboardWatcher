use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ContentType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardEntry {
    pub id: Option<i64>,
    pub content_type: ContentType,
    pub image_path: Option<String>,
    pub text_content: Option<String>,
    pub created_at: String,
}

impl ClipboardEntry {
    pub fn new_text_entry(text: String) -> Self {
        Self {
            id: None,
            content_type: ContentType::Text,
            text_content: Some(text),
            image_path: None,
            created_at: chrono::Utc::now().to_string(),
        }
    }

    pub fn new_image_entry(image_path: String) -> Self {
        Self {
            id: None,
            content_type: ContentType::Image,
            image_path: Some(image_path),
            text_content: None,
            created_at: chrono::Utc::now().to_string(),
        }
    }
}

pub struct ClipboardDatabase {
    conn: Connection,
}

impl ClipboardDatabase {
    /// Creates a new database connection and initializes the schema
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let db = ClipboardDatabase { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Creates the clipboard_history table if it doesn't exist
    fn init_schema(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type TEXT NOT NULL, -- TEXT, IMAGE
                image_path TEXT,
                text_content TEXT,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        // Create index on created_at for faster sorting
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON clipboard_history(created_at DESC)",
            [],
        )?;

        Ok(())
    }

    /// Saves a clipboard entry to the database
    pub fn save_entry(&self, clipboard_entry: ClipboardEntry) -> Result<i64>
    {
        match clipboard_entry.content_type {
            ContentType::Text => {
                self.conn.execute(
                    "INSERT INTO clipboard_history (content_type, text_content, created_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params!["TEXT", &clipboard_entry.text_content, &clipboard_entry.created_at],
                )?;
            }
            ContentType::Image => {
                self.conn.execute(
                    "INSERT INTO clipboard_history (content_type, image_path, created_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params!["IMAGE", &clipboard_entry.image_path, &clipboard_entry.created_at],
                )?;
            }
        }
        Ok(self.conn.last_insert_rowid())
    }

    /// Retrieves all clipboard entries, sorted by most recent first
    pub fn get_all_entries(&self) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, text_content, image_path, created_at FROM clipboard_history ORDER BY created_at DESC"
        )?;

        let entries = stmt.query_map([], |row| {
            let id: i64 = row.get(0)?;
            let content_type_str: String = row.get(1)?;
            let content_type = if content_type_str == "TEXT" {
                ContentType::Text
            } else {
                ContentType::Image
            };

            Ok(ClipboardEntry {
                id: Some(id),
                content_type,
                text_content: row.get(2)?,
                image_path: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        entries.collect()
    }

    /// Retrieves the latest N clipboard entries
    pub fn get_recent_entries(&self, limit: usize) -> Result<Vec<ClipboardEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, text_content, image_path, created_at FROM clipboard_history ORDER BY created_at DESC LIMIT ?1"
        )?;

        let entries = stmt.query_map([limit], |row| {
            let content_type_str: String = row.get(0)?;
            let content_type = if content_type_str == "TEXT" {
                ContentType::Text
            } else {
                ContentType::Image
            };

            Ok(ClipboardEntry {
                id: row.get(0)?,
                content_type,
                text_content: row.get(1)?,
                image_path: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        entries.collect()
    }

    /// Deletes an entry by ID
    pub fn delete_entry(&self, id: i64) -> Result<i64> {
        match self.conn.execute(
            "DELETE FROM clipboard_history WHERE id = ?1",
            [id],
        ) {
            Ok(_) => Ok(id),
            Err(e) => Err(e),
        }
    }

    /// Clears all clipboard history
    pub fn clear_all(&self) -> Result<()> {
        self.conn.execute("DELETE FROM clipboard_history", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> ClipboardDatabase {
        // Use in-memory database for tests to avoid file permission issues
        let conn = Connection::open_in_memory().unwrap();
        let db = ClipboardDatabase { conn };
        db.init_schema().unwrap();
        db
    }

    fn cleanup_test_db() {
        // No cleanup needed for in-memory database
    }

    #[test]
    fn test_database_creation() {
        let db = create_test_db();
        // If we get here, database was created successfully
        assert!(true);
        cleanup_test_db();
    }

    #[test]
    fn test_save_and_retrieve_entry() {
        let db = create_test_db();

        // Save an entry
        let content = "Test clipboard content".to_string();
        let entry = ClipboardEntry::new_text_entry(content.clone());
        let id = db.save_entry(entry).unwrap();
        assert!(id > 0);

        // Retrieve all entries
        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text_content, Some(content));

        cleanup_test_db();
    }

    #[test]
    fn test_multiple_entries_ordering() {
        let db = create_test_db();

        // Save multiple entries
        db.save_entry(ClipboardEntry::new_text_entry("First".to_string())).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.save_entry(ClipboardEntry::new_text_entry("Second".to_string())).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.save_entry(ClipboardEntry::new_text_entry("Third".to_string())).unwrap();

        // Retrieve and check ordering (most recent first)
        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].text_content, Some("Third".to_string()));
        assert_eq!(entries[1].text_content, Some("Second".to_string()));
        assert_eq!(entries[2].text_content, Some("First".to_string()));

        cleanup_test_db();
    }

    #[test]
    fn test_get_recent_entries() {
        let db = create_test_db();

        // Save 5 entries
        for i in 1..=5 {
            db.save_entry(ClipboardEntry::new_text_entry(format!("Entry {}", i))).unwrap();
        }

        // Get only 3 most recent
        let entries = db.get_recent_entries(3).unwrap();
        assert_eq!(entries.len(), 3);

        cleanup_test_db();
    }

    #[test]
    fn test_delete_entry() {
        let db = create_test_db();

        let id = db.save_entry(ClipboardEntry::new_text_entry("To be deleted".to_string())).unwrap();
        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 1);

        db.delete_entry(id).unwrap();
        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 0);

        cleanup_test_db();
    }

    #[test]
    fn test_clear_all() {
        let db = create_test_db();

        // Add multiple entries
        db.save_entry(ClipboardEntry::new_text_entry("Entry 1".to_string())).unwrap();
        db.save_entry(ClipboardEntry::new_text_entry("Entry 2".to_string())).unwrap();
        db.save_entry(ClipboardEntry::new_text_entry("Entry 3".to_string())).unwrap();

        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 3);

        // Clear all
        db.clear_all().unwrap();
        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 0);

        cleanup_test_db();
    }
}
