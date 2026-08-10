
use rodio::{OutputStream, Sink, source::SineWave, Source};
use std::time::Duration;

fn main() {
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&handle).unwrap();
    sink.append(SineWave::new(440.0).take_duration(Duration::from_secs(3)).amplify(0.20));
    sink.sleep_until_end();
    println!("Done playing!");
}

