// /home/marcos/novprojeto/player/src/ui/app.rs

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crossbeam_channel::{Sender as CrossbeamSender, Receiver as CrossbeamReceiver, TryRecvError as CrossbeamTryRecvError};
use std::time::{Duration, Instant};

use eframe::egui;
use rodio::Sink;
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use rand::Rng;

// --- Enum para Modos de Repetição ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum RepeatMode {
    Off,       // Sem repetição
    Playlist,  // Repetir a playlist inteira
    Track,     // Repetir a faixa atual
}

// Implementa a lógica de ciclo para o botão
impl RepeatMode {
    fn next(&self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::Playlist,
            RepeatMode::Playlist => RepeatMode::Track,
            RepeatMode::Track => RepeatMode::Off,
        }
    }

    // Ícone usado no botão e na mensagem de status
    fn icon(&self) -> &'static str {
        match self {
            RepeatMode::Off => "🔁 Off",
            RepeatMode::Playlist => "🔁 All",
            RepeatMode::Track => "🔁¹ One",
        }
    }
}
// --- Fim do Enum ---

// --- Mensagens de Comunicação ---
#[derive(Debug, Clone)]
pub enum AudioCommand {
    PlayTrack(PathBuf, usize),
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(Duration),
}

#[derive(Debug)]
pub enum AudioResponse {
    LoadError(PathBuf, String),
    PlaybackStarted,
    PlaybackPaused,
    PlaybackStopped,
    PlaybackEnded,
    CurrentlyPlaying(Option<usize>, Option<Duration>),
    SeekCompleted(Duration),
}


// --- Estado da Aplicação ---
#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct MusicPlayerApp {
    playlist: Vec<PathBuf>,
    current_track_index: Option<usize>,
    selected_track_index: Option<usize>,
    volume: f32,
    is_shuffle: bool,
    repeat_mode: RepeatMode,

    // --- Adicionar estado para a janela "Sobre" ---
    #[serde(skip)] // Não precisa salvar o estado da janela
    show_about_window: bool,
    // --- Fim da adição ---

    #[serde(skip)]
    current_track_duration: Option<Duration>,
    #[serde(skip)]
    playback_start_time: Option<Instant>,
    #[serde(skip)]
    elapsed_duration_at_pause: Duration,

    // --- Campos não persistidos ---
    #[serde(skip)]
    _stream_handle: Option<rodio::OutputStreamHandle>,
    #[serde(skip)]
    sink: Arc<Mutex<Sink>>,
    #[serde(skip)]
    status: String,
    #[serde(skip)]
    error_message: Option<String>,
    #[serde(skip)]
    audio_command_sender: Option<CrossbeamSender<AudioCommand>>,
    #[serde(skip)]
    audio_response_receiver: Option<CrossbeamReceiver<AudioResponse>>,
    #[serde(skip)]
    is_loading: bool,
    #[serde(skip)]
    loading_file_path: Option<PathBuf>,
    #[serde(skip)]
    is_playing: bool,
    #[serde(skip)]
    is_paused: bool,
}

// --- Default impl ---
impl Default for MusicPlayerApp {
    fn default() -> Self {
        Self {
            _stream_handle: None,
            sink: Arc::new(Mutex::new(Sink::new_idle().0)),
            status: "Initializing...".to_string(),
            error_message: None,
            audio_command_sender: None,
            audio_response_receiver: None,
            is_loading: false,
            loading_file_path: None,
            playlist: Vec::new(),
            current_track_index: None,
            selected_track_index: None,
            is_playing: false,
            is_paused: false,
            volume: 0.5,
            is_shuffle: false,
            repeat_mode: RepeatMode::Off,
            show_about_window: false, // Janela começa fechada
            current_track_duration: None,
            playback_start_time: None,
            elapsed_duration_at_pause: Duration::ZERO,
        }
    }
}

// --- Métodos de MusicPlayerApp ---
impl MusicPlayerApp {
    pub fn setup(
        &mut self,
        stream_handle: rodio::OutputStreamHandle,
        sink: Arc<Mutex<Sink>>,
        sender: CrossbeamSender<AudioCommand>,
        receiver: CrossbeamReceiver<AudioResponse>,
    ) {
        self._stream_handle = Some(stream_handle);
        self.sink = sink;
        self.audio_command_sender = Some(sender);
        self.audio_response_receiver = Some(receiver);
        self.send_audio_command(AudioCommand::SetVolume(self.volume));
        self.status = if self.playlist.is_empty() { "Ready. Add files to the playlist.".to_string() } else { "Playlist loaded. Ready.".to_string() };
        if let Some(idx) = self.current_track_index { if idx >= self.playlist.len() { self.current_track_index = None; } }
        if let Some(idx) = self.selected_track_index { if idx >= self.playlist.len() { self.selected_track_index = None; } }
        self.is_playing = false; self.is_paused = false; self.reset_progress_state(); self.current_track_index = None;
    }

    fn send_audio_command(&mut self, command: AudioCommand) {
        if let Some(sender) = &self.audio_command_sender {
            println!("GUI sending command: {:?}", command);
            if let Err(e) = sender.send(command) {
                self.status = "Fatal Error: Audio thread disconnected.".to_string();
                self.error_message = Some(format!("Failed to send command: {}", e));
                eprintln!("GUI send error: {}", e);
                self.audio_command_sender = None;
                self.audio_response_receiver = None;
                self.reset_playback_state();
            }
        } else {
            if self.error_message.is_none() || !self.status.starts_with("Fatal Error") {
                self.status = "Error: Audio command sender not available.".to_string();
                eprintln!("GUI Error: Audio command sender not initialized or disconnected.");
            }
        }
    }

    fn get_filename(&self, path: &PathBuf) -> String {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string())
    }

    fn play_random_track(&mut self) {
        if self.playlist.is_empty() {
            self.status = "Playlist is empty.".to_string();
            self.reset_playback_state();
            return;
        }
        let mut rng = rand::thread_rng();
        let playlist_len = self.playlist.len();
        let mut random_index = rng.gen_range(0..playlist_len);
        if playlist_len > 1 && Some(random_index) == self.current_track_index {
             random_index = (random_index + rng.gen_range(1..playlist_len)) % playlist_len;
        }
        self.play_track_at_index(random_index);
    }

    fn play_track_at_index(&mut self, index: usize) {
         if let Some(path_ref) = self.playlist.get(index) {
            let path_to_play = path_ref.clone();
            self.status = format!("Requesting play: {}", self.get_filename(&path_to_play));
            self.is_loading = true;
            self.loading_file_path = Some(path_to_play.clone());
            self.reset_progress_state();
            self.send_audio_command(AudioCommand::PlayTrack(path_to_play, index));
        } else {
            self.status = format!("Error: Could not find track at index {}", index);
            eprintln!("Error: play_track_at_index called with invalid index {}", index);
            self.reset_playback_state();
        }
    }

    fn play_next_track(&mut self) {
        if self.playlist.is_empty() {
            self.status = "Playlist is empty.".to_string();
            self.reset_playback_state();
            return;
        }
        if self.is_shuffle {
            println!("play_next_track (Shuffle ON): Playing random track.");
            self.play_random_track();
        } else {
            let current_idx = self.current_track_index.unwrap_or(self.playlist.len());
            let next_index = current_idx + 1;
            if next_index < self.playlist.len() {
                println!("play_next_track (Sequential): Playing next track at index {}", next_index);
                self.play_track_at_index(next_index);
            } else {
                if self.repeat_mode == RepeatMode::Playlist {
                    println!("play_next_track (Sequential, Repeat Playlist): Wrapping around to index 0.");
                    self.play_track_at_index(0);
                } else {
                    println!("play_next_track (Sequential, Repeat Off/Track): Playlist finished.");
                    self.status = "Playlist finished.".to_string();
                    self.reset_playback_state();
                }
            }
        }
    }

    fn play_previous_track(&mut self) {
        if self.playlist.is_empty() {
             self.status = "Playlist is empty.".to_string();
             return;
        }
        let prev_index = self.current_track_index
            .and_then(|idx| idx.checked_sub(1))
            .unwrap_or_else(|| self.playlist.len() - 1);
        println!("play_previous_track: Playing track at index {}", prev_index);
        self.play_track_at_index(prev_index);
    }

    fn reset_progress_state(&mut self) {
        self.current_track_duration = None;
        self.playback_start_time = None;
        self.elapsed_duration_at_pause = Duration::ZERO;
    }

    fn reset_playback_state(&mut self) {
        self.is_playing = false;
        self.is_paused = false;
        self.current_track_index = None;
        self.reset_progress_state();
        self.is_loading = false;
        self.loading_file_path = None;
        self.error_message = None;
    }

    fn calculate_elapsed(&self) -> Duration {
        if self.is_playing {
            self.elapsed_duration_at_pause + self.playback_start_time.map_or(Duration::ZERO, |start| start.elapsed())
        } else {
            self.elapsed_duration_at_pause
        }
    }

    fn format_duration(duration: Duration) -> String {
        let total_secs = duration.as_secs();
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{:02}:{:02}", mins, secs)
    }

    // --- Função para mostrar a janela "Sobre" ---
    fn show_about_window(&mut self, ctx: &egui::Context) {
        // Cria uma nova janela egui
        egui::Window::new("Sobre Rust Music Player")
            .open(&mut self.show_about_window) // Controla a visibilidade com o estado `show_about_window`
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Rust Music Player lITE");
                ui.label("Versão: Alfa 1.1 "); 
                ui.separator();
                ui.label("Desenvolvido por:");
                ui.label("Marcos da Silva"); 
                ui.separator();
                ui.label("Contato:");
                ui.label("marcossl10@hotmail.com");
                ui.label("Me paga um cafe?");
                ui.label("Pix 83980601072");
                ui.horizontal(|ui| {
                    ui.label("GitHub:");
                    // <- SEU LINK DO GITHUB AQUI (texto e URL)
                    ui.hyperlink_to("Marcos", "");
                });
                // Adicione mais links/contatos se desejar
                // ui.horizontal(|ui| {
                //     ui.label("Website:");
                //     ui.hyperlink_to("meusite.com", "https://meusite.com");
                // });
                ui.separator();
                ui.label("Feito com Rust, egui, rodio, symphonia e lofty.");
                ui.separator();
                // Botão para fechar (alternativa ao 'X' da janela)
                if ui.button("Fechar").clicked() {
                    // A janela será fechada automaticamente na próxima frame
                    // devido ao `.open(&mut self.show_about_window)`
                }
            });
    }
}

// --- Implementação eframe::App ---
impl eframe::App for MusicPlayerApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
        println!("App state saved.");
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Processar Respostas de Áudio ---
        let mut responses_to_process = Vec::new();
        if let Some(receiver) = &self.audio_response_receiver {
            loop {
                match receiver.try_recv() {
                    Ok(response) => responses_to_process.push(response),
                    Err(CrossbeamTryRecvError::Empty) => break,
                    Err(CrossbeamTryRecvError::Disconnected) => {
                        self.status = "Fatal Error: Audio thread disconnected.".to_string();
                        self.error_message = Some("Audio processing thread terminated unexpectedly.".to_string());
                        eprintln!("GUI: Audio thread disconnected.");
                        self.audio_response_receiver = None;
                        self.audio_command_sender = None;
                        self.reset_playback_state();
                        responses_to_process.clear();
                        break;
                    }
                }
            }
        }

        for response in responses_to_process {
            println!("GUI processing collected response: {:?}", response);
            match response {
                AudioResponse::LoadError(path, err_msg) => {
                    if self.loading_file_path.as_ref() == Some(&path) {
                        self.is_loading = false;
                        self.loading_file_path = None;
                    }
                    self.error_message = Some(format!("Failed to load '{}': {}", path.display(), err_msg));
                    self.status = "Error loading file".to_string();
                    if self.current_track_index.map_or(false, |idx| self.playlist.get(idx) == Some(&path)) {
                        self.reset_playback_state();
                    }
                }
                AudioResponse::PlaybackStarted => {
                    self.is_playing = true;
                    self.is_paused = false;
                    self.is_loading = false;
                    self.loading_file_path = None;
                    self.error_message = None;
                    self.playback_start_time = Some(Instant::now());
                    if let Some(idx) = self.current_track_index {
                        if let Some(path) = self.playlist.get(idx) {
                            self.status = format!("Playing: {}", self.get_filename(path));
                        }
                    } else { self.status = "Playing...".to_string(); }
                }
                AudioResponse::PlaybackPaused => {
                    if self.is_playing {
                        self.is_playing = false;
                        self.is_paused = true;
                        self.status = "Paused".to_string();
                        self.error_message = None;
                        if let Some(start) = self.playback_start_time.take() {
                            self.elapsed_duration_at_pause += start.elapsed();
                        }
                    }
                }
                AudioResponse::PlaybackStopped => {
                    self.status = "Stopped".to_string();
                    self.reset_playback_state();
                }
                AudioResponse::PlaybackEnded => {
                    println!("GUI: Playback ended detected.");
                    if let Some(last_played_index) = self.current_track_index {
                        match self.repeat_mode {
                            RepeatMode::Track => {
                                println!("Repeat Track: Replaying index {}", last_played_index);
                                self.play_track_at_index(last_played_index);
                            }
                            RepeatMode::Playlist => {
                                println!("Repeat Playlist: Playing next track (shuffle={})", self.is_shuffle);
                                self.play_next_track();
                            }
                            RepeatMode::Off => {
                                if self.is_shuffle {
                                    println!("Repeat Off (Shuffle ON): Playing next random track");
                                    self.play_random_track();
                                } else {
                                    if last_played_index >= self.playlist.len().saturating_sub(1) {
                                        println!("Repeat Off (Sequential): Playlist finished.");
                                        self.status = "Playlist finished.".to_string();
                                        self.reset_playback_state();
                                    } else {
                                        println!("Repeat Off (Sequential): Playing next track");
                                        self.play_next_track();
                                    }
                                }
                            }
                        }
                    } else {
                        println!("PlaybackEnded received but no current_track_index known. Stopping.");
                        self.reset_playback_state();
                    }
                }
                AudioResponse::CurrentlyPlaying(index_option, duration_option) => {
                    self.current_track_index = index_option;
                    self.current_track_duration = duration_option;
                    self.is_loading = false;
                    self.loading_file_path = None;
                    if let Some(idx) = index_option {
                        if let Some(path) = self.playlist.get(idx) {
                            self.status = format!("Playing: {}", self.get_filename(path));
                            self.error_message = None;
                            self.is_playing = true;
                            self.is_paused = false;
                            self.selected_track_index = Some(idx);
                            self.elapsed_duration_at_pause = Duration::ZERO;
                            self.playback_start_time = Some(Instant::now());
                        } else {
                            self.status = format!("Error: Playing unknown track at index {}", idx);
                            self.error_message = Some(format!("Playlist desync? Index {} not found.", idx));
                            eprintln!("Playlist desync: Audio thread reported playing index {} but playlist has {} items.", idx, self.playlist.len());
                            self.reset_playback_state();
                        }
                    } else {
                        if self.is_playing || self.is_paused {
                            self.reset_playback_state();
                            self.status = "Stopped".to_string();
                        }
                    }
                }
                AudioResponse::SeekCompleted(new_elapsed_time) => {
                    println!("GUI: Received SeekCompleted confirmation: {:?}", new_elapsed_time);
                    self.elapsed_duration_at_pause = new_elapsed_time;
                    if self.is_playing { self.playback_start_time = Some(Instant::now()); }
                    else { self.playback_start_time = None; }
                    ctx.request_repaint();
                }
            }
        }

        // --- Adicionar Menu Superior ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // Removido o menu "Arquivo" como discutido
                // ui.menu_button("Arquivo", |ui| { ... });

                ui.menu_button("Ajuda", |ui| {
                    if ui.button("Sobre...").clicked() {
                        self.show_about_window = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // --- Mostrar a Janela "Sobre" (se o estado for true) ---
        if self.show_about_window {
            self.show_about_window(ctx); // Chama a função corrigida
        }

        // --- Layout da UI Principal (Revertido para o Original) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Rust Music Player Alfa 1.1");
            ui.separator();

            // --- Barra de Progresso/Seek e Tempos ---
            let elapsed = self.calculate_elapsed();
            let total = self.current_track_duration.unwrap_or(Duration::ZERO);
            let mut seek_to_fraction: Option<f32> = None;
            ui.vertical(|ui| {
                let slider_enabled = (self.is_playing || self.is_paused) && total > Duration::ZERO && self.audio_command_sender.is_some();
                let current_progress = if total > Duration::ZERO { (elapsed.as_secs_f32() / total.as_secs_f32()).clamp(0.0, 1.0) } else { 0.0 };
                let mut slider_value = current_progress;
                let progress_slider = ui.add_enabled(
                    slider_enabled,
                    egui::Slider::new(&mut slider_value, 0.0..=1.0)
                        .show_value(false)
                        .min_decimals(3)
                        .step_by(0.001)
                );
                if progress_slider.changed() { seek_to_fraction = Some(slider_value); }
                progress_slider.on_hover_text(format!("{} / {} (Click or Drag to Seek)", Self::format_duration(elapsed), Self::format_duration(total)));
                ui.horizontal(|ui| {
                    ui.label(Self::format_duration(elapsed));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(Self::format_duration(total));
                    });
                });
            });
            ui.separator();

            if let Some(fraction) = seek_to_fraction {
                if total > Duration::ZERO {
                    let target_secs = total.as_secs_f64() * fraction as f64;
                    let seek_duration = Duration::from_secs_f64(target_secs);
                    println!("GUI: Requesting Seek to: {:?} (fraction: {})", seek_duration, fraction);
                    self.send_audio_command(AudioCommand::Seek(seek_duration));
                    self.elapsed_duration_at_pause = seek_duration;
                    if self.is_playing { self.playback_start_time = Some(Instant::now()); }
                    else { self.playback_start_time = None; }
                    ctx.request_repaint();
                }
            }

            // --- Controles de Reprodução (Layout Original) ---
            ui.horizontal(|ui| {
                 let can_interact_playback = !self.is_loading && !self.playlist.is_empty() && self.audio_command_sender.is_some();
                 let can_stop = (self.is_playing || self.is_paused) && self.audio_command_sender.is_some();
                 let can_previous = !self.is_loading && self.audio_command_sender.is_some() && !self.playlist.is_empty();
                 let can_next = !self.is_loading && self.audio_command_sender.is_some() && !self.playlist.is_empty();

                 let play_pause_text = if self.is_playing { "Pause ❚❚" } else { "Play ▶" };
                 let play_pause_button = ui.add_enabled(can_interact_playback, egui::Button::new(play_pause_text).min_size(egui::vec2(60.0, 0.0)));
                 if play_pause_button.clicked() {
                     if self.is_playing { self.send_audio_command(AudioCommand::Pause); }
                     else {
                         if self.is_paused { self.send_audio_command(AudioCommand::Play); }
                         else {
                             let index_to_play = self.selected_track_index.filter(|&idx| idx < self.playlist.len()).unwrap_or(0);
                             if !self.playlist.is_empty() { self.play_track_at_index(index_to_play); }
                         }
                     }
                 }

                 let stop_button = ui.add_enabled(can_stop, egui::Button::new("Stop ⏹"));
                 if stop_button.clicked() { self.send_audio_command(AudioCommand::Stop); }

                 let previous_button = ui.add_enabled(can_previous, egui::Button::new("⏮ Prev"));
                 if previous_button.clicked() { self.play_previous_track(); }

                 let next_button = ui.add_enabled(can_next, egui::Button::new("Next ⏭"));
                 if next_button.clicked() { self.play_next_track(); }

                 let shuffle_text = if self.is_shuffle { "🔀 ON" } else { "🔀 OFF" };
                 let shuffle_button = ui.add_enabled(can_interact_playback, egui::Button::new(shuffle_text).selected(self.is_shuffle));
                 if shuffle_button.clicked() {
                     self.is_shuffle = !self.is_shuffle;
                     self.status = format!("Shuffle mode: {}", if self.is_shuffle { "ON" } else { "OFF" });
                 }
                 shuffle_button.on_hover_text(format!("Turn Shuffle {}", if self.is_shuffle { "OFF" } else { "ON" }));

                 let repeat_icon = self.repeat_mode.icon();
                 let repeat_button = ui.add_enabled(
                     can_interact_playback,
                     egui::Button::new(repeat_icon).selected(self.repeat_mode != RepeatMode::Off)
                 );
                 if repeat_button.clicked() {
                     self.repeat_mode = self.repeat_mode.next();
                     self.status = format!("Repeat mode: {}", self.repeat_mode.icon());
                 }
                 repeat_button.on_hover_text(format!("Cycle Repeat Mode (Current: {})", self.repeat_mode.icon()));

            });

            // --- Controle de Volume (Layout Original) ---
            ui.horizontal(|ui| {
                ui.label("Volume:");
                let volume_slider = ui.add(egui::Slider::new(&mut self.volume, 0.0..=1.0).show_value(false));
                ui.label(format!("{:3.0}%", self.volume * 100.0)).on_hover_text("Volume");
                if volume_slider.changed() {
                    self.send_audio_command(AudioCommand::SetVolume(self.volume));
                }
            });
            ui.separator();

             // --- Controles da Playlist (Layout Original) ---
             ui.horizontal(|ui| {
                 let can_manage_playlist = !self.is_loading && self.audio_command_sender.is_some();
                 let add_button = ui.add_enabled(can_manage_playlist, egui::Button::new("➕ Add"));
                 if add_button.clicked() {
                     if let Some(paths) = FileDialog::new().add_filter("Audio Files", &["mp3", "wav", "ogg", "flac"]).pick_files() {
                         if !paths.is_empty() {
                             let num_added = paths.len();
                             let was_empty = self.playlist.is_empty();
                             self.playlist.extend(paths);
                             self.status = format!("Added {} file(s) to playlist.", num_added);
                             self.error_message = None;
                             if was_empty { self.selected_track_index = Some(0); }
                         } else {
                             if self.error_message.is_none() { self.status = "File selection cancelled.".to_string(); }
                         }
                     }
                 }
                 let can_clear = !self.playlist.is_empty() && can_manage_playlist;
                 let clear_button = ui.add_enabled(can_clear, egui::Button::new("🗑️ Clear"));
                 if clear_button.clicked() {
                     if self.is_playing || self.is_paused { self.send_audio_command(AudioCommand::Stop); }
                     self.playlist.clear();
                     self.selected_track_index = None;
                     self.status = "Playlist cleared.".to_string();
                     self.reset_playback_state();
                 }
                 let can_remove = self.selected_track_index.is_some() && can_manage_playlist;
                 let remove_button = ui.add_enabled(can_remove, egui::Button::new("➖ Remove"));
                 if remove_button.clicked() {
                     if let Some(index_to_remove) = self.selected_track_index {
                         if index_to_remove < self.playlist.len() {
                             let was_playing_removed = self.current_track_index == Some(index_to_remove);
                             if was_playing_removed { self.send_audio_command(AudioCommand::Stop); }
                             let removed_path = self.playlist.remove(index_to_remove);
                             self.status = format!("Removed: {}", self.get_filename(&removed_path));
                             self.current_track_index = self.current_track_index.and_then(|current_idx| {
                                 if current_idx == index_to_remove { None }
                                 else if current_idx > index_to_remove { Some(current_idx - 1) }
                                 else { Some(current_idx) }
                             });
                             self.selected_track_index = if self.playlist.is_empty() { None }
                             else if index_to_remove >= self.playlist.len() { Some(self.playlist.len() - 1) }
                             else { Some(index_to_remove) };
                             if was_playing_removed { self.reset_playback_state(); }
                         }
                     }
                 }
             });
            ui.separator();

            // --- Exibição da Playlist (Layout Original) ---
            ui.label("Playlist:");
            let mut play_clicked_index: Option<usize> = None;
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(ui.available_height() * 0.5)
                .show(ui, |ui| {
                    if self.playlist.is_empty() {
                        ui.weak("(Empty)");
                    } else {
                        let current_track_idx_display = self.current_track_index;
                        let is_playing_display = self.is_playing;
                        let is_paused_display = self.is_paused;
                        for (index, path) in self.playlist.iter().enumerate() {
                            let filename = self.get_filename(path);
                            let is_current_track = current_track_idx_display == Some(index);
                            let item_text = if is_current_track && is_playing_display {
                                format!("▶ {}", filename)
                            } else if is_current_track && is_paused_display {
                                format!("⏸ {}", filename)
                            } else {
                                format!("  {}", filename)
                            };
                            let response = ui.selectable_label(self.selected_track_index == Some(index), item_text);
                            if response.clicked() {
                                self.selected_track_index = Some(index);
                                let should_play = match (current_track_idx_display, is_playing_display, is_paused_display) {
                                    (Some(current_idx), true, false) if current_idx == index => false,
                                    (Some(current_idx), false, true) if current_idx == index => true,
                                    _ => true,
                                };
                                if should_play { play_clicked_index = Some(index); }
                            }
                            response.on_hover_text(path.display().to_string());
                        }
                    }
                });

            if let Some(index_to_play) = play_clicked_index {
                self.play_track_at_index(index_to_play);
            }
            ui.separator();

            // --- Indicador de Carregamento, Status e Erro (Layout Original) ---
            if self.is_loading {
                ui.horizontal(|ui| {
                    ui.spinner();
                    if let Some(path) = &self.loading_file_path {
                        ui.weak(format!("Loading {}...", self.get_filename(path)));
                    } else {
                        ui.weak("Loading...");
                    }
                });
            }
            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            } else {
                ui.weak(&self.status); // Status normal em cinza
            }

            // --- Solicitar Repaint ---
            if self.is_loading || self.is_playing {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        });
    }
}
