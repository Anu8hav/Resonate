#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod db;
mod scanner;
mod audio_engine;

use audio_engine::AudioEngine;

use db::{DbState, AlbumDto, TrackDto};
use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tauri::{Manager, Emitter};
use tauri_plugin_dialog::DialogExt;

/// Tracks the active polling loop to prevent stale concurrent loops from emitting events.
static POLLING_GENERATION: AtomicUsize = AtomicUsize::new(0);

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
    // Run the blocking file system and database operations in a separate thread
    tauri::async_runtime::spawn_blocking(move || {
        let path = Path::new(&folder_path);

        // Step 1: Scan the directory for audio files
        let scan_result = scanner::scan_directory(path)?;

        // Step 2: Persist results to the database
        let db_state = app.state::<DbState>();
        let conn = db_state.0.lock().map_err(|e| format!("DB lock error: {}", e))?;

        let mut albums_seen = std::collections::HashSet::new();
        let mut artists_seen = std::collections::HashSet::new();
        let mut kept_file_paths = std::collections::HashSet::new();

        for track in &scan_result.tracks {
            // Get or create the artist
            let artist_id = db::get_or_create_artist(&conn, &track.artist)?;
            artists_seen.insert(artist_id.clone());

            // Get or create the album (if present)
            let album_id = match &track.album {
                Some(album_title) => {
                    // Note: This relies on the SELECT logic in get_or_create_album or handles constraint errors.
                    let aid = db::get_or_create_album(&conn, album_title, &artist_id)?;
                    albums_seen.insert(aid.clone());
                    Some(aid)
                }
                None => None,
            };

            // Upsert the track
            db::upsert_track(&conn, track, &artist_id, album_id.as_deref())?;
            kept_file_paths.insert(track.file_path.clone());
        }

        // Step 3: Remove missing tracks from this folder
        db::delete_missing_tracks(&conn, &folder_path, &kept_file_paths)?;

        // Step 4: Record the scan folder
        db::upsert_scan_folder(&conn, &folder_path)?;

        Ok(ScanSummary {
            tracks_found: scan_result.tracks.len() as u32,
            tracks_skipped: scan_result.skipped.len() as u32,
            albums_found: albums_seen.len() as u32,
            artists_found: artists_seen.len() as u32,
        })
    })
    .await
    .map_err(|e| format!("Task failed: {}", e))?
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

/* ── Audio Playback Commands ─────────────────────────────────────────── */

#[tauri::command]
async fn play_track(
    app: tauri::AppHandle,
    file_path: String,
    state: tauri::State<'_, Mutex<Option<AudioEngine>>>,
) -> Result<f64, String> {
    // 1. Play the track
    let duration = {
        let mut engine_opt = state.lock().map_err(|e| format!("Audio engine lock failed: {}", e))?;
        if let Some(engine) = engine_opt.as_mut() {
            // We don't have duration from frontend yet natively passed in this signature unless we change it.
            // Wait, the prompt says: "returns the track's total duration in seconds (read via symphonia/lofty... 
            // reuse metadata already extracted... check if duration is already stored)". 
            // Since the frontend knows the duration from `Track`, let's just accept duration as an arg if needed, 
            // or query it from DB. Let's query from DB!
            
            let db_state = app.state::<DbState>();
            let db_duration = if let Ok(conn) = db_state.0.lock() {
                let dur: Option<u32> = conn.query_row(
                    "SELECT duration_seconds FROM tracks WHERE file_path = ?1",
                    rusqlite::params![file_path],
                    |row| row.get(0),
                ).ok();
                dur.map(|d| d as f64)
            } else {
                None
            };
            
            engine.play_file(&file_path, db_duration)?;
            db_duration.unwrap_or(0.0) // Return the duration
        } else {
            return Err("No audio output device available.".into());
        }
    };

    // 2. Start position polling loop
    let generation = POLLING_GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;

            // Check if this loop is still the active one
            if POLLING_GENERATION.load(Ordering::SeqCst) != generation {
                break;
            }

            let state = app_clone.state::<Mutex<Option<AudioEngine>>>();
            let mut is_finished = false;
            let mut position = None;

            if let Ok(engine_opt) = state.lock() {
                if let Some(engine) = engine_opt.as_ref() {
                    is_finished = engine.is_finished();
                    position = engine.get_position();
                }
            }

            if is_finished {
                let _ = app_clone.emit("track-ended", ());
                break;
            } else if let Some(pos) = position {
                let _ = app_clone.emit("playback-position", pos);
            }
        }
    });

    Ok(duration)
}

#[tauri::command]
fn pause_track(state: tauri::State<'_, Mutex<Option<AudioEngine>>>) -> Result<(), String> {
    let mut engine_opt = state.lock().map_err(|e| format!("Lock failed: {}", e))?;
    if let Some(engine) = engine_opt.as_mut() {
        engine.pause();
        Ok(())
    } else {
        Err("No audio device available.".into())
    }
}

#[tauri::command]
fn resume_track(state: tauri::State<'_, Mutex<Option<AudioEngine>>>) -> Result<(), String> {
    let mut engine_opt = state.lock().map_err(|e| format!("Lock failed: {}", e))?;
    if let Some(engine) = engine_opt.as_mut() {
        engine.resume();
        Ok(())
    } else {
        Err("No audio device available.".into())
    }
}

#[tauri::command]
fn stop_track(state: tauri::State<'_, Mutex<Option<AudioEngine>>>) -> Result<(), String> {
    // Stop the polling loop
    POLLING_GENERATION.fetch_add(1, Ordering::SeqCst);
    
    let mut engine_opt = state.lock().map_err(|e| format!("Lock failed: {}", e))?;
    if let Some(engine) = engine_opt.as_mut() {
        engine.stop();
        Ok(())
    } else {
        Err("No audio device available.".into())
    }
}

#[tauri::command]
async fn seek_track(
    app: tauri::AppHandle,
    position_seconds: f64,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Mutex<Option<AudioEngine>>>();
        let mut engine_opt = state.lock().map_err(|e| format!("Lock failed: {}", e))?;
        if let Some(engine) = engine_opt.as_mut() {
            engine.seek(position_seconds)
        } else {
            Err("No audio device available.".into())
        }
    })
    .await
    .map_err(|e| format!("Seek task failed: {}", e))?
}

#[tauri::command]
fn set_volume(
    volume: f32,
    state: tauri::State<'_, Mutex<Option<AudioEngine>>>,
) -> Result<(), String> {
    let mut engine_opt = state.lock().map_err(|e| format!("Lock failed: {}", e))?;
    if let Some(engine) = engine_opt.as_mut() {
        engine.set_volume(volume);
        Ok(())
    } else {
        Err("No audio device available.".into())
    }
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

            // Initialize AudioEngine and store as Option so we don't panic on missing devices
            let engine = match AudioEngine::new() {
                Ok(e) => Some(e),
                Err(err) => {
                    eprintln!("[audio_engine] Failed to initialize: {}", err);
                    None
                }
            };
            app.manage(Mutex::new(engine)); // Wrapped in Option<AudioEngine>

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pick_music_folder,
            scan_library,
            get_all_albums,
            get_all_tracks,
            play_track,
            pause_track,
            resume_track,
            stop_track,
            seek_track,
            set_volume,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
