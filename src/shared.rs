use std::fmt;
use std::io::BufReader;
use std::{fs::OpenOptions, io::Write, path::Path};

use rodio::OutputStream;
use std::fs::File;

/// tries to write to debug.log, doesn't fail or return an error if it can't.
pub fn log_debug(text: &str) {
    let result = OpenOptions::new()
        .append(true)
        .create(true)
        .open(Path::new("debug.log"));

    if let Ok(mut file) = result {
        let _ = writeln!(file, "{text}");
    }
}

/// round target to num_places decimal places
pub fn round_precise(target: f64, num_places: usize) -> f64 {
    let mult = (10.0 as f64).powf(num_places as f64);
    (target * mult).round() / mult
}

/// Plays sound. Needs to be kept alive for the sounds to finish playing, so
/// create once as part of an App.
pub struct AudioPlayer {
    /// sound stops playing when we drop this, so we give it the same lifetime as us.
    stream_handle: OutputStream,
}

impl AudioPlayer {
    pub fn new() -> Self {
        // Get an output stream handle to the default physical sound device.
        // Note that the playback stops when the stream_handle is dropped.
        let mut stream_handle =
            rodio::OutputStreamBuilder::open_default_stream().expect("open default audio stream");
        stream_handle.log_on_drop(false);
        AudioPlayer {
            stream_handle: stream_handle,
        }
    }

    fn play_sound(&self, sound_path: &str) {
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open(sound_path).unwrap());
        // the playback stops when the sink is dropped unless we call .detach()
        rodio::play(&self.stream_handle.mixer(), file)
            .unwrap()
            .detach();
    }

    pub fn play_sound_1(&self) {
        self.play_sound("assets/audio/back_001.ogg");
    }

    pub fn play_sound_2(&self) {
        self.play_sound("assets/audio/confirmation_001.ogg");
    }

    pub fn play_sound_3(&self) {
        self.play_sound("assets/audio/confirmation_002.ogg");
    }

    pub fn play_sound_4(&self) {
        self.play_sound("assets/audio/jingles_NES00.ogg");
    }
}

impl fmt::Debug for AudioPlayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioPlayer")
            .field("stream_handle", &"")
            .finish()
    }
}
