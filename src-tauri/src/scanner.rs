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
    pub total_tracks: Option<u32>,
    pub format: String,
    pub bitrate: Option<u32>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u8>,
    pub channels: Option<u8>,
    pub cover_path: Option<String>,
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

/// Normalize a file path for consistent database storage and comparison.
/// Converts backslashes to forward slashes, and on Windows, converts to lowercase.
pub fn normalize_path(path: &Path) -> String {
    let mut s = path.to_string_lossy().to_string().replace('\\', "/");
    #[cfg(target_os = "windows")]
    {
        s = s.to_lowercase();
    }
    s
}

/// Scan a directory recursively for audio files and extract metadata from each.
///
/// This function does NOT write to the database — it only reads files and returns
/// structured metadata. Persistence is handled separately by the caller.
///
/// Individual file failures (corrupt files, unsupported codecs) are caught and
/// logged — the file path is added to `skipped` and scanning continues.
pub fn scan_directory(path: &Path, app_data_dir: &Path) -> Result<ScanResult, String> {
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
        match read_track_metadata(file_path, app_data_dir) {
            Ok(track) => tracks.push(track),
            Err(err) => {
                eprintln!("[scanner] Skipping {:?}: {}", file_path_str, err);
                skipped.push(file_path_str);
            }
        }
    }

    Ok(ScanResult { tracks, skipped })
}

/// Read metadata from a single audio file using lofty, and extract cover art.
pub fn read_track_metadata(path: &Path, app_data_dir: &Path) -> Result<ScannedTrack, String> {
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
    let total_tracks = tag.and_then(|t| t.track_total());

    // Extract cover art
    let mut cover_path = None;
    if let Some(t) = &tag {
        let pictures = t.pictures();
        if !pictures.is_empty() {
            let picture = pictures
                .iter()
                .find(|p| p.pic_type() == lofty::picture::PictureType::CoverFront)
                .unwrap_or(&pictures[0]);

            let data = picture.data();
            let ext = match picture.mime_type() {
                Some(lofty::picture::MimeType::Png) => "png",
                Some(lofty::picture::MimeType::Bmp) => "bmp",
                Some(lofty::picture::MimeType::Gif) => "gif",
                Some(lofty::picture::MimeType::Tiff) => "tiff",
                _ => "jpg", // fallback for Jpeg and others
            };

            use std::hash::{DefaultHasher, Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            data.hash(&mut hasher);
            let hash = hasher.finish();

            let covers_dir = app_data_dir.join("covers");
            if !covers_dir.exists() {
                let _ = std::fs::create_dir_all(&covers_dir);
            }

            let filename = format!("{:016x}.{}", hash, ext);
            let file_path = covers_dir.join(&filename);

            if !file_path.exists() {
                let _ = std::fs::write(&file_path, data);
            }

            cover_path = Some(file_path.to_string_lossy().to_string());
        }
    }

    // Extract audio properties
    let properties = tagged_file.properties();
    let duration_seconds = properties.duration().as_secs() as u32;
    let bitrate = properties.overall_bitrate();
    let sample_rate = properties.sample_rate();
    let bit_depth = properties.bit_depth();
    let channels = properties.channels();

    let format = format_from_extension(path);
    let file_path = normalize_path(path);

    Ok(ScannedTrack {
        title,
        artist,
        album,
        duration_seconds,
        file_path,
        track_number,
        total_tracks,
        format,
        bitrate,
        sample_rate,
        bit_depth,
        channels,
        cover_path,
    })
}
