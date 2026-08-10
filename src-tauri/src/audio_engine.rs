use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

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

    /// Stops any current sink, opens the file, decodes, and starts playback.
    pub fn play_file(&mut self, file_path: &str, duration: Option<f64>) -> Result<(), String> {
        // Stop current playback
        self.stop();

        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open audio file {}: {}", file_path, e))?;
        let reader = BufReader::new(file);
        
        let source = Decoder::new(reader)
            .map_err(|e| format!("Failed to decode audio file: {}", e))?;

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
        if let Some(sink) = &mut self.sink {
            let target_duration = Duration::from_secs_f64(position_seconds);
            
            // First, try rodio's native try_seek which relies on the symphonia decoder's seeking
            match sink.try_seek(target_duration) {
                Ok(_) => {
                    self.accumulated_position = position_seconds;
                    if !sink.is_paused() {
                        self.play_started_at = Some(Instant::now());
                    }
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("[audio_engine] Native try_seek failed: {}, falling back to re-decode", e);
                }
            }

            // Fallback: stop sink, re-decode file, skip_duration (which rapidly consumes decoded samples), append to new sink.
            let path = self.current_track_path.clone().ok_or("No track path for seek fallback")?;
            let was_paused = sink.is_paused();
            let current_vol = sink.volume();

            let file = File::open(&path).map_err(|e| format!("Seek fallback open failed: {}", e))?;
            let reader = BufReader::new(file);
            let source = Decoder::new(reader).map_err(|e| format!("Seek fallback decode failed: {}", e))?;

            // Apply skip_duration to consume samples up to the target
            let skipped_source = source.skip_duration(target_duration);

            let new_sink = Sink::try_new(&self.stream_handle).map_err(|e| format!("Seek fallback sink failed: {}", e))?;
            new_sink.set_volume(current_vol);
            new_sink.append(skipped_source);

            if was_paused {
                new_sink.pause();
            } else {
                new_sink.play();
                self.play_started_at = Some(Instant::now());
            }

            self.sink = Some(new_sink);
            self.accumulated_position = position_seconds;

            Ok(())
        } else {
            Err("No active sink.".into())
        }
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
        self.sink.as_ref().map(|s| s.empty()).unwrap_or(true)
    }
}
