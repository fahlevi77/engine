use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Trait for simple persistence stores that save full snapshots.
pub trait PersistenceStore: Send + Sync {
    fn save(&self, siddhi_app_id: &str, revision: &str, snapshot: &[u8]);
    fn load(&self, siddhi_app_id: &str, revision: &str) -> Option<Vec<u8>>;
    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String>;
    fn clear_all_revisions(&self, siddhi_app_id: &str);
}

/// Trait for incremental persistence stores.
pub trait IncrementalPersistenceStore: Send + Sync {
    fn save(&self, revision: &str, snapshot: &[u8]);
    fn load(&self, revision: &str) -> Option<Vec<u8>>;
    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String>;
    fn clear_all_revisions(&self, siddhi_app_id: &str);
}

/// Very small in-memory implementation useful for tests.
#[derive(Default)]
pub struct InMemoryPersistenceStore {
    inner: Mutex<HashMap<String, HashMap<String, Vec<u8>>>>,
    last_revision: Mutex<HashMap<String, String>>,
}

impl InMemoryPersistenceStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PersistenceStore for InMemoryPersistenceStore {
    fn save(&self, siddhi_app_id: &str, revision: &str, snapshot: &[u8]) {
        let mut m = self.inner.lock().unwrap();
        let entry = m.entry(siddhi_app_id.to_string()).or_default();
        entry.insert(revision.to_string(), snapshot.to_vec());
        self.last_revision
            .lock()
            .unwrap()
            .insert(siddhi_app_id.to_string(), revision.to_string());
    }

    fn load(&self, siddhi_app_id: &str, revision: &str) -> Option<Vec<u8>> {
        self.inner
            .lock()
            .unwrap()
            .get(siddhi_app_id)
            .and_then(|m| m.get(revision).cloned())
    }

    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String> {
        self.last_revision
            .lock()
            .unwrap()
            .get(siddhi_app_id)
            .cloned()
    }

    fn clear_all_revisions(&self, siddhi_app_id: &str) {
        self.inner.lock().unwrap().remove(siddhi_app_id);
        self.last_revision.lock().unwrap().remove(siddhi_app_id);
    }
}

impl IncrementalPersistenceStore for InMemoryPersistenceStore {
    fn save(&self, revision: &str, snapshot: &[u8]) {
        // For tests we treat incremental same as full with siddhi_app_id="default"
        <Self as PersistenceStore>::save(self, "default", revision, snapshot);
    }

    fn load(&self, revision: &str) -> Option<Vec<u8>> {
        <Self as PersistenceStore>::load(self, "default", revision)
    }

    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String> {
        PersistenceStore::get_last_revision(self, siddhi_app_id)
    }

    fn clear_all_revisions(&self, siddhi_app_id: &str) {
        PersistenceStore::clear_all_revisions(self, siddhi_app_id)
    }
}

// Simple file-based persistence store storing each snapshot as a file
use std::fs;
use std::path::{Path, PathBuf};

pub struct FilePersistenceStore {
    base: PathBuf,
    last_revision: Mutex<HashMap<String, String>>,
}

impl FilePersistenceStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> std::io::Result<Self> {
        let p = path.into();
        fs::create_dir_all(&p)?;
        Ok(Self {
            base: p,
            last_revision: Mutex::new(HashMap::new()),
        })
    }

    fn file_path(&self, app: &str, rev: &str) -> PathBuf {
        self.base.join(app).join(rev)
    }
}

impl PersistenceStore for FilePersistenceStore {
    fn save(&self, siddhi_app_id: &str, revision: &str, snapshot: &[u8]) {
        let dir = self.base.join(siddhi_app_id);
        if let Err(e) = fs::create_dir_all(&dir) {
            eprintln!("FilePersistenceStore: cannot create dir: {}", e);
            return;
        }
        let path = self.file_path(siddhi_app_id, revision);
        match fs::write(&path, snapshot) {
            Ok(_) => {
                self.last_revision
                    .lock()
                    .unwrap()
                    .insert(siddhi_app_id.to_string(), revision.to_string());
            }
            Err(e) => {
                eprintln!("FilePersistenceStore: write failed: {}", e);
            }
        }
    }

    fn load(&self, siddhi_app_id: &str, revision: &str) -> Option<Vec<u8>> {
        let path = self.file_path(siddhi_app_id, revision);
        fs::read(path).ok()
    }

    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String> {
        self.last_revision
            .lock()
            .unwrap()
            .get(siddhi_app_id)
            .cloned()
    }

    fn clear_all_revisions(&self, siddhi_app_id: &str) {
        let dir = self.base.join(siddhi_app_id);
        let _ = fs::remove_dir_all(&dir);
        self.last_revision.lock().unwrap().remove(siddhi_app_id);
    }
}

use rusqlite::{params, Connection};

pub struct SqlitePersistenceStore {
    conn: Mutex<Connection>,
}

impl SqlitePersistenceStore {
    pub fn new<P: AsRef<Path>>(path: P) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS snapshots (app TEXT, rev TEXT, data BLOB, PRIMARY KEY(app, rev))",
            [],
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

impl PersistenceStore for SqlitePersistenceStore {
    fn save(&self, siddhi_app_id: &str, revision: &str, snapshot: &[u8]) {
        let conn = self.conn.lock().unwrap();
        if let Err(e) = conn.execute(
            "INSERT OR REPLACE INTO snapshots(app, rev, data) VALUES (?1, ?2, ?3)",
            params![siddhi_app_id, revision, snapshot],
        ) {
            eprintln!("SqlitePersistenceStore: write failed: {}", e);
        }
    }

    fn load(&self, siddhi_app_id: &str, revision: &str) -> Option<Vec<u8>> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT data FROM snapshots WHERE app=?1 AND rev=?2",
            params![siddhi_app_id, revision],
            |row| row.get(0),
        )
        .ok()
    }

    fn get_last_revision(&self, siddhi_app_id: &str) -> Option<String> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT rev FROM snapshots WHERE app=?1 ORDER BY rowid DESC LIMIT 1",
            params![siddhi_app_id],
            |row| row.get(0),
        )
        .ok()
    }

    fn clear_all_revisions(&self, siddhi_app_id: &str) {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM snapshots WHERE app=?1", params![siddhi_app_id]);
    }
}
