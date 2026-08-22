//! Custom `rodio::Source` implementation backed directly by symphonia's
//! `FormatReader` and `Decoder`, exposing native format-level seeking
//! (using the container's seek index/table) instead of the decode-and-
//! discard approach used by `rodio::Decoder` + `skip_duration()`.

use std::fs::File;
use std::path::Path;

use symphonia::core::audio::{SampleBuffer, SignalSpec};
use symphonia::core::codecs::{self, DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// A `rodio::Source`-compatible audio source that wraps symphonia's
/// `FormatReader` directly, providing fast index-based seeking via
/// `FormatReader::seek()`.
pub struct SymphoniaSource {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn codecs::Decoder>,
    track_id: u32,
    sample_buffer: Option<SampleBuffer<f32>>,
    sample_index: usize,
    spec: SignalSpec,
    total_duration_secs: Option<f64>,
}

impl SymphoniaSource {
    /// Opens an audio file, probes its container format, and initialises
    /// the decoder for the first supported audio track.
    pub fn new(path: &Path) -> Result<Self, String> {
        let file = File::open(path).map_err(|e| format!("Failed to open file: {e}"))?;
        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| format!("Probe failed: {e}"))?;

        let format_reader = probed.format;

        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or("No supported audio track found")?
            .clone();

        let track_id = track.id;

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("Decoder init failed: {e}"))?;

        // Build an initial SignalSpec from codec params — will be
        // overwritten with the actual spec on first decoded packet.
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track
            .codec_params
            .channels
            .unwrap_or(symphonia::core::audio::Channels::FRONT_LEFT
                | symphonia::core::audio::Channels::FRONT_RIGHT);
        let spec = SignalSpec::new(sample_rate, channels);

        // Compute total duration in seconds from n_frames if available.
        let total_duration_secs = track.codec_params.n_frames.map(|n| {
            n as f64 / sample_rate as f64
        });

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            sample_buffer: None,
            sample_index: 0,
            spec,
            total_duration_secs,
        })
    }

    /// Seeks to the given position using the format's native seek
    /// capability (index/seek-table based, NOT sample-by-sample).
    ///
    /// After seeking, the decoder's internal state is reset so it can
    /// cleanly start decoding from the new position.
    ///
    /// NOTE: On a freshly constructed `SymphoniaSource` that hasn't
    /// decoded any packets yet, `decoder.reset()` is a harmless no-op
    /// (it simply clears internal buffers that are already empty).
    pub fn seek_to(&mut self, position_seconds: f64) -> Result<(), String> {
        let seek_to = SeekTo::Time {
            time: Time::from(position_seconds),
            track_id: Some(self.track_id),
        };

        self.format_reader
            .seek(SeekMode::Accurate, seek_to)
            .map_err(|e| format!("Seek failed: {e}"))?;

        // Reset decoder state after seeking — clears any partially
        // buffered codec data from the pre-seek position.
        self.decoder.reset();

        // Discard any buffered samples from the previous position.
        self.sample_buffer = None;
        self.sample_index = 0;

        Ok(())
    }

    /// Decodes the next packet from the format reader and fills the
    /// internal sample buffer. Returns `None` on EOF or unrecoverable
    /// error (treated as stream end).
    fn next_packet_samples(&mut self) -> Option<()> {
        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(p) => p,
                Err(_) => return None, // EOF or error → stream end
            };

            // Skip packets for other tracks (e.g. video in an MP4).
            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let num_frames = decoded.capacity();

                    let mut buf = SampleBuffer::<f32>::new(
                        num_frames as u64,
                        spec,
                    );
                    buf.copy_interleaved_ref(decoded);

                    self.spec = spec;
                    self.sample_buffer = Some(buf);
                    self.sample_index = 0;
                    return Some(());
                }
                Err(symphonia::core::errors::Error::DecodeError(_)) => {
                    // Skip corrupt/bad packets, try the next one.
                    continue;
                }
                Err(_) => return None, // Unrecoverable error → stream end
            }
        }
    }
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        loop {
            if let Some(buf) = &self.sample_buffer {
                if self.sample_index < buf.samples().len() {
                    let sample = buf.samples()[self.sample_index];
                    self.sample_index += 1;
                    return Some(sample);
                }
            }
            // Current buffer exhausted — decode next packet.
            self.next_packet_samples()?;
        }
    }
}

impl rodio::Source for SymphoniaSource {
    fn current_frame_len(&self) -> Option<usize> {
        self.sample_buffer
            .as_ref()
            .map(|b| b.samples().len() - self.sample_index)
    }

    fn channels(&self) -> u16 {
        self.spec.channels.count() as u16
    }

    fn sample_rate(&self) -> u32 {
        self.spec.rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        self.total_duration_secs
            .map(|s| std::time::Duration::from_secs_f64(s))
    }
}
