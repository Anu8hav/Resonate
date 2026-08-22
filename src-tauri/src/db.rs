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

    // Explicitly DISABLE foreign keys before migrations to allow table recreation
    // (some SQLite builds default to ON). We enable them after migration.
    conn.execute_batch("PRAGMA foreign_keys=OFF;")
        .map_err(|e| format!("Failed to disable foreign keys: {}", e))?;

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
            total_tracks INTEGER,
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
    let _ = conn.execute("ALTER TABLE tracks ADD COLUMN bit_depth INTEGER", []);
    let _ = conn.execute("ALTER TABLE tracks ADD COLUMN channels INTEGER", []);

    // Check schema version for migrations
    let user_version: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| format!("Failed to read user_version: {}", e))?;

    if user_version < 1 {
        // Version 1 Migration: Enforce UNIQUE constraints properly on existing tables by deduping rows.
        // Because SQLite's IF NOT EXISTS ignores schema changes on existing tables, we must explicitly
        // recreate them to apply the UNIQUE constraints and clean up the existing duplicate rows.
        conn.execute_batch(
            r#"
            BEGIN TRANSACTION;
            
            -- Artists migration
            CREATE TABLE artists_new (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            INSERT OR IGNORE INTO artists_new
            SELECT * FROM artists;
            DROP TABLE artists;
            ALTER TABLE artists_new RENAME TO artists;

            -- Albums migration
            CREATE TABLE albums_new (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                cover_path TEXT,
                total_tracks INTEGER,
                FOREIGN KEY (artist_id) REFERENCES artists(id),
                UNIQUE(title, artist_id)
            );
            INSERT OR IGNORE INTO albums_new
            SELECT * FROM albums;
            DROP TABLE albums;
            ALTER TABLE albums_new RENAME TO albums;

            -- Tracks migration
            CREATE TABLE tracks_new (
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
            
            -- Group by normalized path: replace backslashes and lowercase.
            -- This perfectly mimics the `scanner::normalize_path` logic in SQL.
            INSERT INTO tracks_new
            SELECT * FROM tracks
            WHERE rowid IN (
                SELECT MAX(rowid) FROM tracks
                GROUP BY LOWER(REPLACE(file_path, '\', '/'))
            );
            DROP TABLE tracks;
            ALTER TABLE tracks_new RENAME TO tracks;

            PRAGMA user_version = 1;
            COMMIT TRANSACTION;
            "#
        ).map_err(|e| format!("Failed to migrate to version 1: {}", e))?;
    }

    if user_version < 2 {
        // Version 2 Migration: Add total_tracks to albums
        conn.execute_batch(
            r#"
            BEGIN TRANSACTION;
            ALTER TABLE albums ADD COLUMN total_tracks INTEGER;
            PRAGMA user_version = 2;
            COMMIT TRANSACTION;
            "#
        ).map_err(|e| format!("Failed to migrate to version 2: {}", e))?;
    }

    // Enable foreign keys now that migration is complete
    conn.execute_batch("PRAGMA foreign_keys=ON;")
        .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

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
    cover_path: Option<&str>,
    incoming_total_tracks: Option<u32>,
) -> Result<String, String> {
    let trimmed = title.trim();

    // Check if album exists
    let existing: Option<(String, Option<u32>)> = conn.query_row(
        "SELECT id, total_tracks FROM albums WHERE LOWER(TRIM(title)) = LOWER(?1) AND artist_id = ?2",
        params![trimmed, artist_id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();

    if let Some((id, existing_total)) = existing {
        // If we found a track with a known total, and the DB doesn't have one or the DB has a smaller one, update it.
        // Prefer the highest value seen, as a fuller scan is more authoritative.
        if let Some(incoming_t) = incoming_total_tracks {
            let should_update = match existing_total {
                None => true,
                Some(et) => incoming_t > et,
            };
            if should_update {
                conn.execute(
                    "UPDATE albums SET total_tracks = ?1 WHERE id = ?2",
                    params![incoming_t, id]
                ).map_err(|e| format!("Failed to update album total_tracks: {}", e))?;
                // log conflict if needed
                if existing_total.is_some() && existing_total.unwrap() != incoming_t {
                    eprintln!("[scanner] Conflict for album '{}': updating total_tracks from {} to {}", trimmed, existing_total.unwrap(), incoming_t);
                }
            }
        }
        return Ok(id);
    }

    // Insert new album
    let id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO albums (id, title, artist_id, cover_path, total_tracks) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, trimmed, artist_id, cover_path, incoming_total_tracks],
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
    let normalized_file_path = crate::scanner::normalize_path(std::path::Path::new(&track.file_path));

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
            normalized_file_path,
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
    pub year: Option<u32>,
    pub source: String,
    pub total_tracks: Option<u32>,
    pub locally_owned_count: u32,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumDetailDto {
    #[serde(flatten)]
    pub album: AlbumDto,
    pub tracks: Vec<TrackDto>,
}

/// Query all albums joined with artist name and track count.
pub fn query_all_albums(conn: &Connection) -> Result<Vec<AlbumDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT al.id, al.title, ar.name, al.cover_path, al.total_tracks,
             (SELECT COUNT(*) FROM tracks WHERE album_id = al.id) as locally_owned_count
             FROM albums al
             JOIN artists ar ON al.artist_id = ar.id
             ORDER BY ar.name COLLATE NOCASE ASC, al.title COLLATE NOCASE ASC"
        )
        .map_err(|e| format!("Failed to prepare query: {}", e))?;

    let albums = stmt
        .query_map([], |row| {
            Ok(AlbumDto {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                year: None,
                source: "local".to_string(),
                total_tracks: row.get(4)?,
                locally_owned_count: row.get(5)?,
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
            "SELECT t.id, t.title, ar.name, al.title, t.duration_seconds, t.file_path, t.format, t.bitrate, t.sample_rate, t.bit_depth, t.channels, al.cover_path
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
                cover_url: row.get(11)?,
                source: "local".to_string(),
                file_path: row.get(5)?,
                format: row.get(6)?,
                bitrate: row.get(7)?,
                sample_rate: row.get(8)?,
                bit_depth: row.get(9)?,
                channels: row.get(10)?,
            })
        })
        .map_err(|e| format!("Failed to query tracks: {}", e))?;

    let mut result = Vec::new();
    for t in tracks {
        result.push(t.map_err(|e| format!("Row error: {}", e))?);
    }
    Ok(result)
}

/// Query a single album with its full tracklist.
pub fn get_album_with_tracks(conn: &Connection, album_id: &str) -> Result<AlbumDetailDto, String> {
    let mut stmt = conn
        .prepare(
            "SELECT al.id, al.title, ar.name, al.cover_path, al.total_tracks,
             (SELECT COUNT(*) FROM tracks WHERE album_id = al.id) as locally_owned_count
             FROM albums al
             JOIN artists ar ON al.artist_id = ar.id
             WHERE al.id = ?1"
        )
        .map_err(|e| format!("Failed to prepare album query: {}", e))?;

    let album: AlbumDto = stmt
        .query_row(params![album_id], |row| {
            Ok(AlbumDto {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                cover_url: row.get(3)?,
                year: None,
                source: "local".to_string(),
                total_tracks: row.get(4)?,
                locally_owned_count: row.get(5)?,
            })
        })
        .map_err(|e| format!("Failed to find album '{}': {}", album_id, e))?;

    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.title, ar.name, al.title, t.duration_seconds, t.cover_path,
             t.file_path, t.format, t.bitrate, t.sample_rate, t.bit_depth, t.channels
             FROM tracks t
             JOIN artists ar ON t.artist_id = ar.id
             LEFT JOIN albums al ON t.album_id = al.id
             WHERE t.album_id = ?1
             ORDER BY t.track_number ASC, t.title COLLATE NOCASE ASC"
        )
        .map_err(|e| format!("Failed to prepare tracks query: {}", e))?;

    let tracks = stmt
        .query_map(params![album_id], |row| {
            Ok(TrackDto {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(5)?,
                source: "local".to_string(),
                file_path: Some(row.get(6)?),
                format: row.get(7)?,
                bitrate: row.get(8)?,
                sample_rate: row.get(9)?,
                bit_depth: row.get(10)?,
                channels: row.get(11)?,
            })
        })
        .map_err(|e| format!("Failed to query tracks: {}", e))?;

    let mut result_tracks = Vec::new();
    for t in tracks {
        result_tracks.push(t.map_err(|e| format!("Row error: {}", e))?);
    }

    Ok(AlbumDetailDto {
        album,
        tracks: result_tracks,
    })
}

pub fn query_track_by_id(conn: &rusqlite::Connection, id: &str) -> Result<Option<TrackDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.title, ar.name, al.title, t.duration_seconds, t.file_path, t.format, t.bitrate, t.sample_rate, t.bit_depth, t.channels, al.cover_path
             FROM tracks t
             JOIN artists ar ON t.artist_id = ar.id
             LEFT JOIN albums al ON t.album_id = al.id
             WHERE t.id = ?1"
        )
        .map_err(|e| format!("Failed to prepare track query: {}", e))?;

    let mut tracks = stmt
        .query_map(rusqlite::params![id], |row| {
            Ok(TrackDto {
                id: row.get(0)?,
                title: row.get(1)?,
                artist: row.get(2)?,
                album: row.get(3)?,
                duration_seconds: row.get(4)?,
                cover_url: row.get(11)?,
                source: "local".to_string(),
                file_path: row.get(5)?,
                format: row.get(6)?,
                bitrate: row.get(7)?,
                sample_rate: row.get(8)?,
                bit_depth: row.get(9)?,
                channels: row.get(10)?,
            })
        })
        .map_err(|e| format!("Failed to query track: {}", e))?;

    Ok(tracks.next().and_then(|r| r.ok()))
}

#[tauri::command]
pub fn delete_track(db: tauri::State<'_, crate::DbState>, track_id: String) -> Result<(), String> {
    let conn = db.0.lock().map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM tracks WHERE id = ?1", rusqlite::params![track_id])
        .map_err(|e| e.to_string())?;
        
    cleanup_orphaned_albums_and_artists(&conn)?;
    Ok(())
}

fn cleanup_orphaned_albums_and_artists(conn: &rusqlite::Connection) -> Result<(), String> {
    // Delete albums that have no remaining tracks
    conn.execute(
        "DELETE FROM albums WHERE id NOT IN (SELECT DISTINCT album_id FROM tracks WHERE album_id IS NOT NULL)",
        [],
    ).map_err(|e| e.to_string())?;
    
    // Delete artists that have no remaining tracks AND no remaining albums
    conn.execute(
        "DELETE FROM artists WHERE id NOT IN (SELECT DISTINCT artist_id FROM tracks) AND id NOT IN (SELECT DISTINCT artist_id FROM albums)",
        [],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        
        conn.execute_batch(
            "PRAGMA foreign_keys=ON;
            CREATE TABLE artists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            );
            CREATE TABLE albums (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                artist_id TEXT NOT NULL,
                cover_path TEXT,
                total_tracks INTEGER,
                FOREIGN KEY (artist_id) REFERENCES artists(id),
                UNIQUE(title, artist_id)
            );
            CREATE TABLE tracks (
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
            );"
        ).unwrap();
        conn
    }

    #[test]
    fn test_artist_and_album_deduplication() {
        let conn = setup_test_db();
        
        // Test artist deduplication
        let artist_id1 = get_or_create_artist(&conn, "Kendrick Lamar").unwrap();
        let artist_id2 = get_or_create_artist(&conn, "kendrick lamar ").unwrap(); // different case and space
        assert_eq!(artist_id1, artist_id2, "Artists should deduplicate case-insensitively");

        // Test album deduplication
        let album_id1 = get_or_create_album(&conn, "Not Like Us", &artist_id1, None, None).unwrap();
        let album_id2 = get_or_create_album(&conn, "not like us", &artist_id1, None, None).unwrap(); // different case
        assert_eq!(album_id1, album_id2, "Albums should deduplicate case-insensitively");
    }

    #[test]
    fn test_track_upsert_deduplication() {
        let conn = setup_test_db();
        let artist_id = get_or_create_artist(&conn, "Artist").unwrap();
        
        // Initial track
        let mut track1 = ScannedTrack {
            title: "Song 1".to_string(),
            artist: "Artist".to_string(),
            album: None,
            duration_seconds: 100,
            file_path: "C:\\Music\\Song.mp3".to_string(),
            track_number: None,
            format: "MP3".to_string(),
            bitrate: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            cover_path: None,
        };
        
        // The scanner is supposed to normalize it before passing it in.
        // Let's test if upsert_track handles it properly.
        let id1 = upsert_track(&conn, &track1, &artist_id, None).unwrap();
        
        // Second track with different case/slash, mimicking an un-normalized string
        let mut track2 = ScannedTrack {
            title: "Song 1".to_string(),
            artist: "Artist".to_string(),
            album: None,
            duration_seconds: 100,
            file_path: "c:/music/song.mp3".to_string(),
            track_number: None,
            format: "MP3".to_string(),
            bitrate: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            cover_path: None,
        };
        
        let id2 = upsert_track(&conn, &track2, &artist_id, None).unwrap();
        
        // Wait, if upsert_track doesn't normalize, they will be different rows.
        let count: u32 = conn.query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0)).unwrap();
        
        // If normalization is applied correctly in upsert_track, count should be 1.
        // I will assert count == 1, which will fail if I haven't fixed it yet.
        assert_eq!(count, 1, "There should only be 1 track in the DB");
        assert_eq!(id1, id2, "Upsert should return the same ID");
    }
}
