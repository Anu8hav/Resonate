use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::path::Path;
use std::time::Instant;

use crate::symphonia_source::SymphoniaSource;

pub struct AudioEngine {
    stream_handle: OutputStreamHandle,
    sink: Option<Sink>,
    pub current_track_path: Option<String>,
    pub current_track_duration: Option<f64>,
    play_started_at: Option<Instant>,
    accumulated_position: f64,
}

impl AudioEngine {
    /// Initialize the default output device and stream via a background thread.
    /// Returns an error if no audio output device is available.
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        
        std::thread::spawn(move || {
            match OutputStream::try_default() {
                Ok((_stream, handle)) => {
                    if tx.send(Ok(handle)).is_ok() {
                        // Park this thread forever to keep `stream` alive.
                        loop {
                            std::thread::park();
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Err(format!("Failed to get default audio output device: {}", e)));
                }
            }
        });

        let stream_handle = rx.recv().map_err(|_| "Audio thread panicked")??;

        Ok(Self {
            stream_handle,
            sink: None,
            current_track_path: None,
            current_track_duration: None,
            play_started_at: None,
            accumulated_position: 0.0,
        })
    }

    /// Stops any current sink, opens the file via symphonia, and starts playback.
    pub fn play_file(&mut self, file_path: &str, duration: Option<f64>) -> Result<(), String> {
        // Stop current playback
        self.stop();

        let source = SymphoniaSource::new(Path::new(file_path))?;

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {}", e))?;

        sink.append(source);
        sink.play();

        self.sink = Some(sink);
        self.current_track_path = Some(file_path.to_string());
        self.current_track_duration = duration;

        self.accumulated_position = 0.0;
        self.play_started_at = Some(Instant::now());

        Ok(())
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            if !sink.is_paused() {
                sink.pause();
                if let Some(start) = self.play_started_at {
                    self.accumulated_position += start.elapsed().as_secs_f64();
                    self.play_started_at = None;
                }
            }
        }
    }

    pub fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                self.play_started_at = Some(Instant::now());
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        self.current_track_path = None;
        self.current_track_duration = None;
        self.play_started_at = None;
        self.accumulated_position = 0.0;
    }

    pub fn set_volume(&mut self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(clamped);
        }
    }

    pub fn seek(&mut self, position_seconds: f64) -> Result<(), String> {
        let path = self.current_track_path.clone()
            .ok_or("No track loaded")?;

        // Capture state from the current sink before destroying it
        let was_playing = self.sink.as_ref().map(|s| !s.is_paused()).unwrap_or(false);
        let current_vol = self.sink.as_ref().map(|s| s.volume()).unwrap_or(1.0);

        // Drop the old sink completely — this stops old buffered audio immediately
        if let Some(old_sink) = self.sink.take() {
            old_sink.stop();
        }

        // Create a fresh source and use symphonia's native format-level seek
        // (index/seek-table based — O(1) regardless of seek distance)
        let mut source = SymphoniaSource::new(Path::new(&path))?;
        source.seek_to(position_seconds)?;

        let new_sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("Seek sink creation failed: {}", e))?;
        new_sink.set_volume(current_vol);
        new_sink.append(source);

        if !was_playing {
            new_sink.pause();
        }

        self.sink = Some(new_sink);
        self.accumulated_position = position_seconds;
        self.play_started_at = if was_playing { Some(Instant::now()) } else { None };

        Ok(())
    }

    pub fn get_position(&self) -> Option<f64> {
        if self.sink.is_some() {
            let mut pos = self.accumulated_position;
            if let Some(start) = self.play_started_at {
                pos += start.elapsed().as_secs_f64();
            }
            Some(pos)
        } else {
            None
        }
    }

    pub fn is_finished(&self) -> bool {
        self.sink.as_ref().map(|s| s.empty()).unwrap_or(false)
    }
}
