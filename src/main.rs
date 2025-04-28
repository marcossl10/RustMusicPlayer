#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;

use lofty::file::AudioFile;
use ui::app::{MusicPlayerApp, AudioCommand, AudioResponse};
use std::fs::File;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::process;

// Remover import não utilizado: Sample
use rodio::{OutputStream, Sink, Source};
use eframe::egui;

use crossbeam_channel::{unbounded, Receiver as CrossbeamReceiver, Sender as CrossbeamSender, RecvTimeoutError};

use symphonia::core::audio::{SignalSpec, SampleBuffer};
use symphonia::core::codecs::{Decoder, DecoderOptions, CodecParameters};
// Importar SeekErrorKind de errors (ainda necessário para o match, mesmo que não usemos Other)
use symphonia::core::errors::Error as SymphoniaError;
// Remover Track não utilizado de formats
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

// --- Struct SymphoniaSource ---
struct SymphoniaSource {
    reader: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track_id: u32,
    spec: SignalSpec,
    track_time_base: Option<TimeBase>,
    buffer: SampleBuffer<f32>,
    current_frame_pos: usize,
}

impl SymphoniaSource {
    fn new(
        reader: Box<dyn FormatReader>,
        decoder: Box<dyn Decoder>,
        track_id: u32,
        codec_params: &CodecParameters,
    ) -> Result<Self, SymphoniaError> {
        let spec = SignalSpec::new(
            codec_params.sample_rate.ok_or(SymphoniaError::Unsupported("Missing sample rate".into()))?,
            codec_params.channels.ok_or(SymphoniaError::Unsupported("Missing channel spec".into()))?,
        );
        let track_time_base = codec_params.time_base;
        let buffer_capacity = 4096;
        let buffer = SampleBuffer::<f32>::new(buffer_capacity as u64, spec);

        Ok(SymphoniaSource {
            reader,
            decoder,
            track_id,
            spec,
            track_time_base,
            buffer,
            current_frame_pos: 0,
        })
    }

    fn try_seek(&mut self, time: Duration) -> Result<Time, SymphoniaError> {
         let total_secs_f64 = time.as_secs_f64();
         let seconds = total_secs_f64.trunc() as u64;
         let frac = total_secs_f64.fract();
         let seek_time = Time::new(seconds, frac);

         let seek_result = self.reader.seek(
             SeekMode::Accurate,
             SeekTo::Time { time: seek_time, track_id: Some(self.track_id) }
         )?;

         self.decoder.reset();
         self.current_frame_pos = self.buffer.len();
         println!("Symphonia seek completed to raw ts: {}", seek_result.actual_ts);

         if let Some(tb) = self.track_time_base {
             let actual_time = tb.calc_time(seek_result.actual_ts);
             Ok(actual_time)
         } else {
             // Usar SymphoniaError::Unsupported quando time_base está faltando
             eprintln!("Seek Error Detail: Missing time base to calculate actual seek time from timestamp {}", seek_result.actual_ts);
             Err(SymphoniaError::Unsupported(
                 "Missing time base to calculate actual seek time",
             ))
         }
    }

    fn decode_next_frame(&mut self) -> Result<bool, SymphoniaError> {
        loop {
            let packet = match self.reader.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    return Ok(false);
                }
                Err(err) => {
                    eprintln!("Error reading next packet: {}", err);
                    return Err(err);
                }
            };

            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    if self.spec != *decoded.spec() {
                        self.spec = *decoded.spec();
                        let buffer_capacity = self.buffer.capacity();
                        self.buffer = SampleBuffer::<f32>::new(buffer_capacity as u64, self.spec);
                        println!("Spec changed during decoding!");
                    }
                    self.buffer.copy_interleaved_ref(decoded);
                    self.current_frame_pos = 0;
                    return Ok(true);
                }
                Err(SymphoniaError::DecodeError(err)) => {
                    eprintln!("Decode error: {}", err);
                    continue;
                }
                Err(err) => {
                    eprintln!("Fatal decode error: {}", err);
                    return Err(err);
                }
            }
        }
    }
}

// --- Implementações Source e Iterator ---
impl Source for SymphoniaSource {
    #[inline] fn current_frame_len(&self) -> Option<usize> { Some(self.buffer.len() - self.current_frame_pos) }
    #[inline] fn channels(&self) -> u16 { self.spec.channels.count() as u16 }
    #[inline] fn sample_rate(&self) -> u32 { self.spec.rate }
    #[inline] fn total_duration(&self) -> Option<Duration> { None } // Mantém Lofty como primário
}
impl Iterator for SymphoniaSource {
    type Item = f32;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame_pos < self.buffer.len() {
            let sample = self.buffer.samples()[self.current_frame_pos];
            self.current_frame_pos += 1;
            Some(sample)
        } else {
            match self.decode_next_frame() {
                Ok(true) => {
                    if self.current_frame_pos < self.buffer.len() {
                         let sample = self.buffer.samples()[self.current_frame_pos];
                         self.current_frame_pos += 1;
                         Some(sample)
                    } else { None }
                }
                Ok(false) => None,
                Err(_) => None,
            }
        }
    }
}

// --- Função Principal ---
fn main() -> Result<(), eframe::Error> {
    let (stream, stream_handle) = match OutputStream::try_default() {
        Ok(tuple) => tuple,
        Err(e) => { eprintln!("Fatal Error: Could not get default audio output stream: {}", e); process::exit(1); }
    };
    let _keep_stream_alive = stream;

    let options = eframe::NativeOptions {
        persist_window: true,
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 500.0]).with_title("Rust Music Player Lite" ),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Music Player Lite",
        options,
        Box::new(move |cc| {
            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => Arc::new(Mutex::new(s)),
                Err(e) => { panic!("Fatal Error: Could not create audio sink: {}", e); }
            };
            let (cmd_tx, cmd_rx): (CrossbeamSender<AudioCommand>, CrossbeamReceiver<AudioCommand>) = unbounded();
            let (resp_tx, resp_rx): (CrossbeamSender<AudioResponse>, CrossbeamReceiver<AudioResponse>) = unbounded();

            let mut app: MusicPlayerApp = if let Some(storage) = cc.storage {
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
            } else {
                MusicPlayerApp::default()
            };

            let sink_clone = Arc::clone(&sink);
            let resp_tx_clone = resp_tx.clone();
            thread::spawn(move || {
                println!("Audio thread started.");
                let mut current_audio_index: Option<usize> = None;
                let mut current_path_buf: Option<PathBuf> = None;

                loop {
                    let command = match cmd_rx.recv_timeout(Duration::from_millis(100)) {
                         Ok(cmd) => Some(cmd),
                         Err(RecvTimeoutError::Timeout) => None,
                         Err(RecvTimeoutError::Disconnected) => { eprintln!("Audio thread: Command channel disconnected. Shutting down."); break; }
                    };

                    { // Check track end
                        let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for end check");
                        if sink_guard.empty() && current_audio_index.is_some() {
                            println!("Audio thread: Detected track end (sink empty) for index {:?}", current_audio_index);
                            current_audio_index = None; current_path_buf = None;
                            if resp_tx_clone.send(AudioResponse::PlaybackEnded).is_err() { eprintln!("Audio thread: Failed to send PlaybackEnded response (UI likely closed)."); break; }
                            continue;
                        }
                    }

                    if let Some(command) = command {
                        match command {
                            AudioCommand::PlayTrack(path_buf, index) => {
                                println!("Audio thread: Received PlayTrack command for index {}", index);
                                let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for PlayTrack");
                                if !sink_guard.empty() { sink_guard.stop(); }

                                let load_result = load_symphonia_source(&path_buf);
                                match load_result {
                                    Ok(new_source) => {
                                        let mut track_duration: Option<Duration> = None;
                                        match lofty::read_from_path(&path_buf) {
                                            Ok(tagged_file) => { track_duration = Some(tagged_file.properties().duration()); }
                                            Err(e) => { eprintln!("Audio thread: Error reading metadata with Lofty: {}", e); }
                                        }
                                        println!("Audio thread: Track duration: {:?}", track_duration);
                                        current_path_buf = Some(path_buf.clone()); current_audio_index = Some(index);
                                        sink_guard.append(new_source); sink_guard.play(); drop(sink_guard);
                                        if resp_tx_clone.send(AudioResponse::PlaybackStarted).is_err() { break; }
                                        if resp_tx_clone.send(AudioResponse::CurrentlyPlaying(Some(index), track_duration)).is_err() { break; }
                                    }
                                    Err(err_msg) => {
                                        drop(sink_guard); eprintln!("Audio thread: Error loading track with Symphonia {:?}: {}", path_buf, err_msg);
                                        current_audio_index = None; current_path_buf = None;
                                        if resp_tx_clone.send(AudioResponse::LoadError(path_buf.clone(), err_msg)).is_err() { break; }
                                        if resp_tx_clone.send(AudioResponse::CurrentlyPlaying(None, None)).is_err() { break; }
                                    }
                                }
                            }
                            AudioCommand::Play => {
                                let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for Play");
                                if !sink_guard.empty() && sink_guard.is_paused() {
                                    sink_guard.play(); drop(sink_guard);
                                    if resp_tx_clone.send(AudioResponse::PlaybackStarted).is_err() { break; }
                                }
                            }
                            AudioCommand::Pause => {
                                let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for Pause");
                                if !sink_guard.is_paused() && !sink_guard.empty() {
                                    sink_guard.pause(); drop(sink_guard);
                                    if resp_tx_clone.send(AudioResponse::PlaybackPaused).is_err() { break; }
                                }
                            }
                            AudioCommand::Stop => {
                                println!("Audio thread: Received Stop command.");
                                { let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for Stop"); if !sink_guard.empty() { sink_guard.stop(); } }
                                current_audio_index = None; current_path_buf = None;
                                if resp_tx_clone.send(AudioResponse::PlaybackStopped).is_err() { break; }
                            }
                            AudioCommand::SetVolume(new_volume) => {
                                let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for SetVolume");
                                sink_guard.set_volume(new_volume.clamp(0.0, 1.0));
                            }
                            AudioCommand::Seek(target_duration) => {
                                println!("Audio thread: Received Seek command (Symphonia) to {:?}", target_duration);
                                if let Some(path_to_reload) = current_path_buf.clone() {
                                    let sink_guard = sink_clone.lock().expect("Audio thread failed to acquire sink lock for Seek");
                                    if !sink_guard.empty() { sink_guard.stop(); }

                                    match load_symphonia_source(&path_to_reload) {
                                        Ok(mut new_source) => {
                                            println!("Audio thread: Reloaded source for seek.");
                                            match new_source.try_seek(target_duration) {
                                                Ok(actual_time) => {
                                                    println!("Audio thread: Symphonia seek successful to actual time: {:?}", actual_time);
                                                    sink_guard.append(new_source); sink_guard.play(); drop(sink_guard);
                                                    let actual_duration = Duration::from_secs_f64(actual_time.seconds as f64 + actual_time.frac);
                                                    if resp_tx_clone.send(AudioResponse::SeekCompleted(actual_duration)).is_err() { eprintln!("Audio thread: Failed to send SeekCompleted response."); break; }
                                                }
                                                Err(seek_err) => {
                                                    drop(sink_guard); eprintln!("Audio thread: Symphonia seek failed within source: {}", seek_err);
                                                    current_audio_index = None; current_path_buf = None;
                                                    if resp_tx_clone.send(AudioResponse::PlaybackStopped).is_err() { break; }
                                                }
                                            }
                                        }
                                        Err(err_msg) => {
                                            drop(sink_guard); eprintln!("Audio thread: Failed to reload source for seek: {}", err_msg);
                                            current_audio_index = None; current_path_buf = None;
                                            if resp_tx_clone.send(AudioResponse::PlaybackStopped).is_err() { break; }
                                        }
                                    }
                                } else { println!("Audio thread: Cannot seek (Symphonia), no current track path known."); }
                            }
                        }
                    }
                }
                println!("Audio thread finished.");
            });

            app.setup(stream_handle, sink, cmd_tx, resp_rx);
            Box::new(app)
        }),
    )
}

// --- Função load_symphonia_source ---
fn load_symphonia_source(file_path: &PathBuf) -> Result<SymphoniaSource, String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) { hint.with_extension(ext); }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &FormatOptions { enable_gapless: true, ..Default::default() }, &MetadataOptions::default())
        .map_err(|e| format!("Failed to probe format: {}", e))?;

    let reader = probed.format;

    let track = reader
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .cloned()
        .ok_or("No suitable audio track found".to_string())?;

    let track_id = track.id;
    let codec_params = track.codec_params;

    let decoder = symphonia::default::get_codecs()
        .make(&codec_params, &DecoderOptions::default())
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    SymphoniaSource::new(reader, decoder, track_id, &codec_params)
        .map_err(|e| format!("Failed to create SymphoniaSource: {}", e))
}
