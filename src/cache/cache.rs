use rusqlite::{params, Connection, Error as RusqliteError};
use std::env;
use std::error::Error;
use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum CacheError {
    Io(io::Error),
    Rusqlite(RusqliteError),
    FileNotFound(PathBuf),
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::Io(err) => write!(f, "IO Error: {}", err),
            CacheError::Rusqlite(err) => write!(f, "SQLite Error: {}", err),
            CacheError::FileNotFound(path) => {
                write!(f, "File not found in cache: {}", path.display())
            }
        }
    }
}

impl Error for CacheError {}

impl From<io::Error> for CacheError {
    fn from(err: io::Error) -> Self {
        CacheError::Io(err)
    }
}

impl From<RusqliteError> for CacheError {
    fn from(err: RusqliteError) -> Self {
        CacheError::Rusqlite(err)
    }
}

#[derive(Debug)]
pub struct Cache {
    conn: Connection,
}

impl Cache {
    /// Create a new Cache instance.
    ///
    /// This will initialize the SQLite database and cache directory.
    pub fn new() -> Result<Self, CacheError> {
        let cache_dir = match env::var("HOME") {
            Ok(home) => PathBuf::from(home).join(".cache").join("undo"),
            Err(_) => PathBuf::from("~/.cache/undo"),
        };
        fs::create_dir_all(&cache_dir).map_err(|e| CacheError::Io(e))?;

        let conn =
            Connection::open(cache_dir.join("cache.db")).map_err(|e| CacheError::Rusqlite(e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS files (
                id INTEGER PRIMARY KEY,
                path TEXT NOT NULL UNIQUE,
                content BLOB,
                permissions INTEGER
            )",
            params![],
        )
        .map_err(|e| CacheError::Rusqlite(e))?;
        Ok(Cache { conn })
    }

    /// Check if a file is being tracked.
    pub fn is_tracked(&self, file_path: &Path) -> Result<bool, CacheError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM files WHERE path = ?")
            .map_err(|e| CacheError::Rusqlite(e))?;
        let mut rows = stmt
            .query(params![file_path.to_string_lossy()])
            .map_err(|e| CacheError::Rusqlite(e))?;
        Ok(rows.next()?.is_some())
    }

    /// Backup a file.
    pub fn backup(&self, file_path: &Path) -> Result<(), CacheError> {
        let content = fs::read(file_path).map_err(|e| CacheError::Io(e))?;
        let metadata = fs::metadata(file_path).map_err(|e| CacheError::Io(e))?;
        let permissions = metadata.permissions().mode();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO files (path, content, permissions) VALUES (?1, ?2, ?3)",
                params![file_path.to_string_lossy(), content, permissions],
            )
            .map_err(|e| CacheError::Rusqlite(e))?;
        Ok(())
    }

    /// Clear the entire cache by deleting all records in the files table.
    pub fn clear(&mut self) -> Result<(), CacheError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| CacheError::Rusqlite(e))?;

        tx.execute("DELETE FROM files", params![])
            .map_err(|e| CacheError::Rusqlite(e))?;

        tx.commit().map_err(|e| CacheError::Rusqlite(e))?;
        Ok(())
    }

    /// Get a list of all files tracked in the cache.
    pub fn list(&self) -> Result<Vec<PathBuf>, CacheError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM files")
            .map_err(|e| CacheError::Rusqlite(e))?;

        let rows = stmt
            .query_map(params![], |row| {
                let path: String = row.get(0)?;
                Ok(PathBuf::from(path))
            })
            .map_err(|e| CacheError::Rusqlite(e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CacheError::Rusqlite(e))
    }

    /// Restore a file and remove it from the cache database.
    pub fn restore(&mut self, file_path: &Path) -> Result<(), CacheError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| CacheError::Rusqlite(e))?;

        let mut stmt = tx
            .prepare("SELECT content, permissions FROM files WHERE path = ?")
            .map_err(|e| CacheError::Rusqlite(e))?;

        let mut rows = stmt
            .query(params![file_path.to_string_lossy()])
            .map_err(|e| CacheError::Rusqlite(e))?;

        let result = if let Some(row) = rows.next()? {
            let content: Vec<u8> = row.get(0).map_err(|e| CacheError::Rusqlite(e))?;
            let permissions: u32 = row.get(1).map_err(|e| CacheError::Rusqlite(e))?;

            let mut file = File::create(file_path).map_err(|e| CacheError::Io(e))?;
            file.write_all(&content).map_err(|e| CacheError::Io(e))?;

            let metadata = fs::metadata(file_path).map_err(|e| CacheError::Io(e))?;
            let mut perms = metadata.permissions();
            perms.set_mode(permissions);
            fs::set_permissions(file_path, perms).map_err(|e| CacheError::Io(e))?;

            tx.execute(
                "DELETE FROM files WHERE path = ?",
                params![file_path.to_string_lossy()],
            )
            .map_err(|e| CacheError::Rusqlite(e))?;

            Ok(())
        } else {
            Err(CacheError::FileNotFound(file_path.to_path_buf()))
        };

        drop(rows);
        drop(stmt);
        tx.commit().map_err(|e| CacheError::Rusqlite(e))?;

        result
    }
}
