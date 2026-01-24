use rodio::Source;
use std::{
    sync::mpsc::Receiver,
    thread,
    time::{Duration, Instant},
};

pub enum AudioCommand {
    Toggle,
    SetBpm(u64),
    Quit,
}

pub fn run_audio_thread(rx: Receiver<AudioCommand>) {
    let (_stream, stream_handle) =
        rodio::OutputStream::try_default().expect("Falha ao abrir saída de áudio");
    let sink = rodio::Sink::try_new(&stream_handle).expect("Falha ao criar o Sink");

    let mut playing = false;
    let mut bpm = 120;
    let mut last_tick = Instant::now();

    loop {
        while let Ok(cmd) = rx.try_recv() {
            match cmd {
                AudioCommand::Toggle => playing = !playing,
                AudioCommand::SetBpm(new_bpm) => bpm = new_bpm,
                AudioCommand::Quit => return,
            }
        }

        if playing {
            let interval = Duration::from_secs_f64(60.0 / bpm as f64);
            if last_tick.elapsed() >= interval {
                let source = rodio::source::SineWave::new(440.0)
                    .take_duration(Duration::from_millis(50))
                    .amplify(0.2);
                sink.append(source);
                last_tick = Instant::now();
            }
        }

        thread::sleep(Duration::from_millis(1));
    }
}
