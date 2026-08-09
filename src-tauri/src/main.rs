#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
mod scanner;

use db::{DbState, AlbumDto, TrackDto};
use std::path::Path;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

/// Summary returned to the frontend after a scan completes.
#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScanSummary {
    pub tracks_found: u32,
    pub tracks_skipped: u32,
    pub albums_found: u32,
    pub artists_found: u32,
}

/// Opens a native folder picker dialog. Returns the selected path or None if cancelled.
#[tauri::command]
async fn pick_music_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let folder = app.dialog().file().blocking_pick_folder();
    match folder {
        Some(path) => Ok(Some(path.into_path().unwrap().to_string_lossy().to_string())),
        None => Ok(None),
    }
}

/// Scan a folder for audio files, extract metadata, and persist to the local SQLite database.
/// Returns a summary of what was found/skipped.
#[tauri::command]
async fn scan_library(
    app: tauri::AppHandle,
    folder_path: String,
) -> Result<ScanSummary, String> {
    let path = Path::new(&folder_path);

    // Step 1: Scan the directory for audio files
    let scan_result = scanner::scan_directory(path)?;

    // Step 2: Persist results to the database
    let db_state = app.state::<DbState>();
    let conn = db_state.0.lock().map_err(|e| format!("DB lock error: {}", e))?;

    let mut albums_seen = std::collections::HashSet::new();
    let mut artists_seen = std::collections::HashSet::new();

    for track in &scan_result.tracks {
        // Get or create the artist
        let artist_id = db::get_or_create_artist(&conn, &track.artist)?;
        artists_seen.insert(artist_id.clone());

        // Get or create the album (if present)
        let album_id = match &track.album {
            Some(album_title) => {
                let aid = db::get_or_create_album(&conn, album_title, &artist_id)?;
                albums_seen.insert(aid.clone());
                Some(aid)
            }
            None => None,
        };

        // Upsert the track
        db::upsert_track(&conn, track, &artist_id, album_id.as_deref())?;
    }

    // Record the scan folder
    db::upsert_scan_folder(&conn, &folder_path)?;

    Ok(ScanSummary {
        tracks_found: scan_result.tracks.len() as u32,
        tracks_skipped: scan_result.skipped.len() as u32,
        albums_found: albums_seen.len() as u32,
        artists_found: artists_seen.len() as u32,
    })
}

/// Query all albums from the local database.
#[tauri::command]
async fn get_all_albums(app: tauri::AppHandle) -> Result<Vec<AlbumDto>, String> {
    let db_state = app.state::<DbState>();
    let conn = db_state.0.lock().map_err(|e| format!("DB lock error: {}", e))?;
    db::query_all_albums(&conn)
}

/// Query all tracks from the local database.
#[tauri::command]
async fn get_all_tracks(app: tauri::AppHandle) -> Result<Vec<TrackDto>, String> {
    let db_state = app.state::<DbState>();
    let conn = db_state.0.lock().map_err(|e| format!("DB lock error: {}", e))?;
    db::query_all_tracks(&conn)
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize the database in the app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;

            let conn = db::init_db(&app_data_dir)
                .map_err(|e| format!("Database initialization failed: {}", e))?;

            // Store the connection as managed state so commands can access it
            app.manage(DbState(Mutex::new(conn)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pick_music_folder,
            scan_library,
            get_all_albums,
            get_all_tracks,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
