
use rodio::{OutputStream, Sink, source::SineWave, Source};
use std::time::Duration;

fn main() {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let (_stream, handle) = OutputStream::try_default().unwrap();
        tx.send(handle).unwrap();
        loop { std::thread::park(); }
    });

    let handle = rx.recv().unwrap();
    let sink = Sink::try_new(&handle).unwrap();
    sink.append(SineWave::new(440.0).take_duration(Duration::from_secs(3)).amplify(0.20));
    sink.sleep_until_end();
    println!("Done playing!");
}

