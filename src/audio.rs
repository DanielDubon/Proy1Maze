use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

pub struct AudioPlayer {
    sink: Arc<Mutex<Sink>>,
    _stream: OutputStream,
    step_file_path: String,
}

impl AudioPlayer {
    pub fn new(music_file: &str, step_file: &str) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        let music_file = BufReader::new(File::open(music_file).unwrap());
        let music_source = Decoder::new(music_file).unwrap();
        sink.append(music_source);
        sink.set_volume(0.5); // Ajustar el volumen de la música

        AudioPlayer {
            sink: Arc::new(Mutex::new(sink)),
            _stream: stream,
            step_file_path: step_file.to_string(),
        }
    }

    pub fn play(&self) {
        self.sink.lock().unwrap().play();
    }

    pub fn stop(&self) {
        self.sink.lock().unwrap().stop();
    }

    pub fn play_step_sound(&self) {
        println!("Reproduciendo sonido de pasos"); // Log para verificar

        let step_file = BufReader::new(File::open(&self.step_file_path).unwrap());
        let step_source = Decoder::new(step_file).unwrap();

        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.set_volume(1.0); // Asegurar que el volumen sea máximo
        sink.append(step_source);

        // Mantener el Sink activo hasta que el sonido se haya reproducido completamente
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2)); // Esperar 2 segundos para asegurarse
            sink.sleep_until_end();
        });
    }
}
