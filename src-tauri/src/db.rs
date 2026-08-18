use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::sync::Mutex;
use uuid::Uuid;

use crate::scanner::ScannedTrack;

/// Thread-safe wrapper around the SQLite connection, managed as Tauri state.
pub struct DbState(pub Mutex<Connection>);

/// Initialize the database: create the file in app_data_dir and run schema migrations.
pub fn init_db(app_data_dir: &PathBuf) -> Result<Connection, String> {
    // Ensure the app data directory exists
    std::fs::create_dir_all(app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;

    let db_path = app_data_dir.join("resonate.db");
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("Failed to open database at {:?}: {}", db_path, e))?;

    // Enable WAL mode for better concurrent read performance
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

    // Enable foreign keys
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

    // Create tables
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS artists (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS albums (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            cover_path TEXT,
            FOREIGN KEY (artist_id) REFERENCES artists(id),
            UNIQUE(title, artist_id)
        );

        CREATE TABLE IF NOT EXISTS tracks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            artist_id TEXT NOT NULL,
            album_id TEXT,
            duration_seconds INTEGER NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            track_number INTEGER,
            format TEXT,
            bitrate INTEGER,
            sample_rate INTEGER,
            bit_depth INTEGER,
            channels INTEGER,
            FOREIGN KEY (artist_id) REFERENCES artists(id),
            FOREIGN KEY (album_id) REFERENCES albums(id)
        );

        CREATE TABLE IF NOT EXISTS scan_folders (
            id TEXT PRIMARY KEY,
            path TEXT NOT NULL UNIQUE,
            last_scanned_at TEXT
        );"
    ).map_err(|e| format!("Failed to create tables: {}", e))?;

    // Migration: add columns that may not exist in older databases.
    // SQLite doesn't support IF NOT EXISTS for ALTER TABLE ADD COLUMN,
    // so we attempt each and silently ignore "duplicate column" errors.
    let _ = conn.execute("ALTER TABLE tracks ADD COLUMN bit_depth INTEGER", []);
    let _ = conn.execute("ALTER TABLE tracks ADD COLUMN channels INTEGER", []);

    Ok(conn)
}

/// Look up an artist by name (case-insensitive, trimmed). Returns existing ID or creates a new row.
pub fn get_or_create_artist(conn: &Connection, name: &str) -> Result<String, String> {
    let trimmed = name.trim();

    // Try to find existing artist (case-insensitive)
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM artists WHERE LOWER(TRIM(name)) = LOWER(?1)",
            params![trimmed],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    // Insert new artist
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO artists (id, name) VALUES (?1, ?2)",
        params![id, trimmed],
    )
    .map_err(|e| format!("Failed to insert artist '{}': {}", trimmed, e))?;

    Ok(id)
}

/// Look up an album by title + artist_id (case-insensitive title, trimmed).
/// Returns existing ID or creates a new row.
pub fn get_or_create_album(
    conn: &Connection,
    title: &str,
    artist_id: &str,
) -> Result<String, String> {
    let trimmed = title.trim();

    // Try to find existing album by this artist (case-insensitive title)
    let existing: Option<String> = conn
        .query_row(
            "SELECT id FROM albums WHERE LOWER(TRIM(title)) = LOWER(?1) AND artist_id = ?2",
            params![trimmed, artist_id],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    // Insert new album
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO albums (id, title, artist_id, cover_path) VALUES (?1, ?2, ?3, NULL)",
        params![id, trimmed, artist_id],
    )
    .map_err(|e| format!("Failed to insert album '{}': {}", trimmed, e))?;

    Ok(id)
}

/// Insert or update a track, keyed on file_path (natural dedup key).
pub fn upsert_track(
    conn: &Connection,
    track: &ScannedTrack,
    artist_id: &str,
    album_id: Option<&str>,
) -> Result<String, String> {
    let new_id = Uuid::new_v4().to_string();

    let id: String = conn.query_row(
        "INSERT INTO tracks (id, title, artist_id, album_id, duration_seconds, file_path, track_number, format, bitrate, sample_rate, bit_depth, channels)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
         ON CONFLICT(file_path) DO UPDATE SET
            title = excluded.title,
            artist_id = excluded.artist_id,
            album_id = excluded.album_id,
            duration_seconds = excluded.duration_seconds,
            track_number = excluded.track_number,
            format = excluded.format,
            bitrate = excluded.bitrate,
            sample_rate = excluded.sample_rate,
            bit_depth = excluded.bit_depth,
            channels = excluded.channels
         RETURNING id",
        params![
            new_id,
            track.title,
            artist_id,
            album_id,
            track.duration_seconds,
            track.file_path,
            track.track_number,
            track.format,
            track.bitrate,
            track.sample_rate,
            track.bit_depth,
            track.channels,
        ],
        |row| row.get(0)
    )
    .map_err(|e| format!("Failed to upsert track '{}': {}", track.title, e))?;

    Ok(id)
}

/// Delete tracks that belong to `folder_path` but are not in `kept_file_paths`.
pub fn delete_missing_tracks(
    conn: &Connection,
    folder_path: &str,
    kept_file_paths: &std::collections::HashSet<String>,
) -> Result<u32, String> {
    let mut stmt = conn
        .prepare("SELECT id, file_path FROM tracks")
        .map_err(|e| format!("Failed to prepare track deletion query: {}", e))?;

    let mut to_delete = Vec::new();
    let rows = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let file_path: String = row.get(1)?;
            Ok((id, file_path))
        })
        .map_err(|e| format!("Failed to query tracks for deletion: {}", e))?;

    let folder_path_obj = std::path::Path::new(folder_path);

    for row in rows {
        if let Ok((id, file_path)) = row {
            let file_path_obj = std::path::Path::new(&file_path);
            if file_path_obj.starts_with(folder_path_obj) {
                if !kept_file_paths.contains(&file_path) {
                    to_delete.push(id);
                }
            }
        }
    }

    let mut deleted_count = 0;
    for id in to_delete {
        conn.execute("DELETE FROM tracks WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete track {}: {}", id, e))?;
        deleted_count += 1;
    }

    Ok(deleted_count)
}

/// Record a scanned folder with the current timestamp.
pub fn upsert_scan_folder(conn: &Connection, path: &str) -> Result<(), String> {
    conn.execute(
        "INSERT INTO scan_folders (id, path, last_scanned_at)
         VALUES (?1, ?2, datetime('now'))
         ON CONFLICT(path) DO UPDATE SET last_scanned_at = datetime('now')",
        params![Uuid::new_v4().to_string(), path],
    )
    .map_err(|e| format!("Failed to upsert scan folder '{}': {}", path, e))?;

    Ok(())
}

/// DTO for albums returned to the frontend — matches the `Album` interface in types.ts.
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub cover_url: Option<String>,
    pub track_count: u32,
    pub source: String,
}

/// Query all albums joined with artist name and track count.
pub fn query_all_albums(conn: &Connection) -> Result<Vec<AlbumDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.title, ar.name, a.cover_path, COUNT(t.id) as track_count
             FROM albums a
             JOIN artists ar ON a.artist_id = ar.id
             LEFT JOIN tracks t ON t.album_id = a.id
             GROUP BY a.id
             ORDER BY a.title COLLATE NOCASE ASC"
        )
        .map_err(|e| format!("Failed to prepare albums query: {}", e))?;

    let albums = stmt
        .query_map([], |row| {
            Ok(AlbumDto {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                track_count: row.get::<_, u32>(4)?,
                source: "local".to_string(),
            })
        })
        .map_err(|e| format!("Failed to query albums: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(albums)
}

/// DTO for tracks returned to the frontend — matches the `Track` interface in types.ts.
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackDto {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_seconds: u32,
    pub cover_url: Option<String>,
    pub source: String,
    pub file_path: Option<String>,
    pub format: Option<String>,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u32>,
    pub channels: Option<u32>,
}

/// Query all tracks joined with artist and album names.
pub fn query_all_tracks(conn: &Connection) -> Result<Vec<TrackDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.title, ar.name, al.title, t.duration_seconds, t.file_path, t.format, t.bitrate, t.sample_rate, t.bit_depth, t.channels
             FROM tracks t
             JOIN artists ar ON t.artist_id = ar.id
             LEFT JOIN albums al ON t.album_id = al.id
             ORDER BY ar.name COLLATE NOCASE ASC, al.title COLLATE NOCASE ASC, t.track_number ASC, t.title COLLATE NOCASE ASC"
        )
        .map_err(|e| format!("Failed to prepare tracks query: {}", e))?;

    let tracks = stmt
        .query_map([], |row| {
            Ok(TrackDto {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: None, // Cover art extraction deferred to a future pass
                source: "local".to_string(),
                file_path: row.get(5)?,
                format: row.get(6)?,
                bitrate: row.get(7)?,
                sample_rate: row.get(8)?,
                bit_depth: row.get(9)?,
                channels: row.get(10)?,
            })
        })
        .map_err(|e| format!("Failed to query tracks: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tracks)
}
