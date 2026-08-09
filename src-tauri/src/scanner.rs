use std::path::Path;
use walkdir::WalkDir;
use lofty::prelude::*;

/// Metadata extracted from a single audio file.
pub struct ScannedTrack {
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_seconds: u32,
    pub file_path: String,
    pub track_number: Option<u32>,
    pub format: String,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
}

/// Result of scanning a directory — successful tracks plus paths of skipped files.
pub struct ScanResult {
    pub tracks: Vec<ScannedTrack>,
    pub skipped: Vec<String>,
}

/// Audio file extensions we recognize (checked case-insensitively).
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "m4a", "ogg"];

/// Check if a file extension is a supported audio format.
fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Derive the format label from a file extension (uppercase).
fn format_from_extension(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string())
}

/// Extract the filename without extension, used as a fallback title.
fn filename_without_extension(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Track")
        .to_string()
}

/// Scan a directory recursively for audio files and extract metadata from each.
///
/// This function does NOT write to the database — it only reads files and returns
/// structured metadata. Persistence is handled separately by the caller.
///
/// Individual file failures (corrupt files, unsupported codecs) are caught and
/// logged — the file path is added to `skipped` and scanning continues.
pub fn scan_directory(path: &Path) -> Result<ScanResult, String> {
    if !path.exists() {
        return Err(format!("Directory does not exist: {:?}", path));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {:?}", path));
    }

    let mut tracks = Vec::new();
    let mut skipped = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();

        // Skip directories and non-audio files
        if !file_path.is_file() || !is_audio_file(file_path) {
            continue;
        }

        let file_path_str = file_path.to_string_lossy().to_string();

        // Attempt to read metadata — skip this file on failure
        match read_track_metadata(file_path) {
            Ok(track) => tracks.push(track),
            Err(err) => {
                eprintln!("[scanner] Skipping {:?}: {}", file_path_str, err);
                skipped.push(file_path_str);
            }
        }
    }

    Ok(ScanResult { tracks, skipped })
}

/// Read metadata from a single audio file using lofty.
fn read_track_metadata(path: &Path) -> Result<ScannedTrack, String> {
    let tagged_file = lofty::read_from_path(path)
        .map_err(|e| format!("Failed to read tags: {}", e))?;

    // Get the primary tag (ID3v2 for MP3, VorbisComments for FLAC, etc.)
    // Fall back to first available tag if primary isn't present
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag());

    // Extract tag fields with sensible fallbacks
    let title = tag
        .and_then(|t| t.title().map(|s| s.to_string()))
        .unwrap_or_else(|| filename_without_extension(path));

    let artist = tag
        .and_then(|t| t.artist().map(|s| s.to_string()))
        .unwrap_or_else(|| "Unknown Artist".to_string());

    let album = tag.and_then(|t| t.album().map(|s| s.to_string()));

    let track_number = tag.and_then(|t| t.track());

    // Extract audio properties
    let properties = tagged_file.properties();
    let duration_seconds = properties.duration().as_secs() as u32;
    let bitrate = properties.overall_bitrate();
    let sample_rate = properties.sample_rate();

    let format = format_from_extension(path);
    let file_path = path.to_string_lossy().to_string();

    Ok(ScannedTrack {
        title,
        artist,
        album,
        duration_seconds,
        file_path,
        track_number,
        format,
        bitrate,
        sample_rate,
    })
}
