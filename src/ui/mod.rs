use std::{
    sync::{Arc, Mutex},
    thread,
};

use eframe::{App, Error, NativeOptions};
use egui::{
    epaint::text::{FontInsert, FontPriority, InsertFontFamily},
    Color32, Context, FontData, FontFamily, ViewportBuilder,
};
use rfd::FileDialog;

use crate::{
    audio::{midi::{LoadResult, MidiManager}, state::SharedAudioState, Audio},
    ui::{
        frame::{
            attributes::Attributes, keyboard::Keyboard, menu::Menu, note_grid::NoteGrid,
            track_list::TrackList, transport::Transport, Frame,
        },
        message_box::get_message_box,
    },
};

pub mod message_box;
mod frame;

pub struct MidiApp {
    audio: Audio,
    shared_state: Arc<Mutex<SharedAudioState>>,
    open_file_name: String,
    start_octave: i8,
    midi_manager: Arc<Mutex<MidiManager>>,
    select_track: Option<usize>,
    solo_track: Option<usize>,
    show_keyboard: bool,
    show_track_list: bool,
    show_attributes: bool,
}

impl MidiApp {
    pub const APP_NAME: &str = "MIDI Editor";

    /// 앱 폰트와 오디오 시스템을 초기화하고 egui 네이티브 창을 띄운다.
    pub fn run() -> Result<(), Error> {
        let options = NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
            centered: true,
            persist_window: true,
            ..NativeOptions::default()
        };

        eframe::run_native(
            MidiApp::APP_NAME,
            options,
            Box::new(|cc| {
                let nanum_font = include_bytes!("../../assets/NanumGothic.ttf");
                let font = FontInsert {
                    name: "Nanum Gothic".to_string(),
                    data: FontData::from_static(nanum_font),
                    families: vec![InsertFontFamily {
                        family: FontFamily::Proportional,
                        priority: FontPriority::Highest,
                    }],
                };
                cc.egui_ctx.add_font(font);

                let shared_state = Arc::new(Mutex::new(SharedAudioState::default()));
                let audio = Audio::new(shared_state.clone())
                    .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> {
                        Box::new(error)
                    })?;

                Ok(Box::new(MidiApp {
                    audio,
                    shared_state,
                    open_file_name: String::new(),
                    start_octave: -1,
                    midi_manager: Arc::new(Mutex::new(MidiManager::default())),
                    select_track: None,
                    solo_track: None,
                    show_keyboard: true,
                    show_track_list: true,
                    show_attributes: true,
                }))
            }),
        )
    }

    /// 백그라운드 로더가 남긴 성공/실패 결과를 메인 스레드 상태에 반영한다.
    fn process_load_results(&mut self) {
        let pending_result = self
            .midi_manager
            .lock()
            .unwrap()
            .take_pending_result();

        match pending_result {
            Some(LoadResult::Success(loaded_song)) => {
                self.open_file_name = loaded_song.file_name;
                self.select_track = None;
                self.solo_track = None;
                self.midi_manager
                    .lock()
                    .unwrap()
                    .apply_song(loaded_song.song);
                self.refresh_playback_data(true);
                get_message_box()
                    .lock()
                    .unwrap()
                    .show("File loaded successfully.");
            }
            Some(LoadResult::Error(error)) => {
                self.open_file_name.clear();
                self.shared_state.lock().unwrap().clear_playback();
                get_message_box().lock().unwrap().error(error);
            }
            None => {}
        }
    }

    /// 현재 Song과 솔로/뮤트 상태를 기준으로 재생 타임라인을 다시 만든다.
    fn refresh_playback_data(&mut self, reset_cursor: bool) {
        let playback_data = self
            .midi_manager
            .lock()
            .unwrap()
            .song()
            .map(|song| song.playback_data(self.audio.sample_rate, self.solo_track));

        if let Some(playback_data) = playback_data {
            self.shared_state
                .lock()
                .unwrap()
                .set_playback_data(playback_data, reset_cursor);
        } else {
            self.shared_state.lock().unwrap().clear_playback();
        }
    }

    /// 파일 선택 대화상자를 열고 실제 파싱은 백그라운드 스레드에서 수행한다.
    fn open_file(&mut self) {
        if self.midi_manager.lock().unwrap().is_loading() {
            return;
        }

        let Some(path) = FileDialog::default()
            .add_filter("MIDI Files", &["mid", "midi"])
            .pick_file()
        else {
            get_message_box().lock().unwrap().error("No file selected.");
            return;
        };

        let file_name = path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled.mid".to_string());

        self.midi_manager.lock().unwrap().begin_loading();

        let midi_manager = Arc::clone(&self.midi_manager);
        thread::spawn(move || {
            let result = MidiManager::load_file(path.as_path());
            let mut midi_manager = midi_manager.lock().unwrap();
            match result {
                Ok(song) => midi_manager.finish_loading(file_name, song),
                Err(error) => midi_manager.finish_loading_error(error.to_string()),
            }
        });
    }

    /// 현재 열린 곡과 재생 상태를 모두 닫는다.
    fn close(&mut self) {
        self.midi_manager.lock().unwrap().close();
        self.shared_state.lock().unwrap().clear_playback();
        self.open_file_name.clear();
        self.select_track = None;
        self.solo_track = None;
    }

    /// 재생 직전에 최신 트랙 설정으로 타임라인을 갱신한 뒤 재생을 시작한다.
    fn play(&mut self) {
        self.refresh_playback_data(false);
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.playback_cursor_samples >= shared_state.song_length_samples {
            shared_state.seek_samples(0);
        }
        shared_state.set_playing(true);
    }

    /// 정지 시에는 재생을 끊고 커서를 처음으로 되돌린다.
    fn stop(&mut self) {
        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.set_playing(false);
        shared_state.seek_samples(0);
    }
}

impl App for MidiApp {
    /// 매 프레임마다 로드 결과를 처리하고 각 패널을 최신 상태로 다시 그린다.
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.process_load_results();

        ctx.style_mut(|style| {
            style.interaction.selectable_labels = false;
        });

        let fill = Color32::from_rgb(0, 0, 0);
        let is_playing = self.shared_state.lock().unwrap().is_playing;
        let is_loading = self.midi_manager.lock().unwrap().is_loading();

        let mut menu = Menu;
        let mut keyboard = Keyboard;
        let mut track_list = TrackList;
        let mut attributes = Attributes;
        let mut transport = Transport;
        let mut note_grid = NoteGrid;

        egui::TopBottomPanel::top(Menu::FRAME_NAME)
            .frame(egui::Frame::new().inner_margin(Menu::INNER_MARGIN).fill(fill))
            .exact_height(Menu::HEIGHT)
            .resizable(Menu::RESIZABLE)
            .show(ctx, |ui| menu.draw(ui, self));

        if self.show_keyboard {
            egui::TopBottomPanel::bottom(Keyboard::FRAME_NAME)
                .frame(egui::Frame::new().inner_margin(Keyboard::INNER_MARGIN).fill(fill))
                .exact_height(Keyboard::HEIGHT)
                .resizable(Keyboard::RESIZABLE)
                .show(ctx, |ui| keyboard.draw(ui, self));
        }

        if self.show_track_list {
            egui::SidePanel::left(TrackList::FRAME_NAME)
                .frame(egui::Frame::new().inner_margin(TrackList::INNER_MARGIN).fill(fill))
                .exact_width(TrackList::WIDTH)
                .resizable(TrackList::RESIZABLE)
                .show(ctx, |ui| track_list.draw(ui, self));
        }

        if self.show_attributes {
            egui::SidePanel::right(Attributes::FRAME_NAME)
                .frame(egui::Frame::new().inner_margin(Attributes::INNER_MARGIN).fill(fill))
                .exact_width(Attributes::WIDTH)
                .resizable(Attributes::RESIZABLE)
                .show(ctx, |ui| attributes.draw(ui, self));
        }

        egui::TopBottomPanel::top(Transport::FRAME_NAME)
            .frame(egui::Frame::new().inner_margin(Transport::INNER_MARGIN).fill(fill))
            .exact_height(Transport::HEIGHT)
            .resizable(Transport::RESIZABLE)
            .show(ctx, |ui| transport.draw(ui, self));

        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(NoteGrid::INNER_MARGIN).fill(fill))
            .show(ctx, |ui| {
                note_grid.draw(ui, self);
                get_message_box().lock().unwrap().draw(ui);
            });

        // 재생 중이거나 로딩 중일 때는 외부 입력이 없어도 UI를 계속 갱신한다.
        if is_playing || is_loading {
            ctx.request_repaint();
        }
    }
}
