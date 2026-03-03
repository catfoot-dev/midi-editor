use std::{collections::HashMap, sync::{Arc, Mutex}, thread};
use eframe::{App, Error, NativeOptions};
use egui::{
    Color32, Context, FontData, FontFamily, ViewportBuilder, epaint::text::{FontInsert, FontPriority, InsertFontFamily}
};
use rfd::FileDialog;

use crate::{audio::{Audio, midi::MidiManager, midi_struct::MidiNote, state::SharedAudioState}, ui::{
    frame::{Frame, attributes::Attributes, keyboard::Keyboard, menu::Menu, note_grid::NoteGrid, track_list::TrackList, transport::Transport},
    message_box::get_message_box,
}};

pub mod message_box;
mod frame;

// 전체 UI 레이아웃 예시
// +-------------------------------------------------------------+
// | [1. TOP] Menu                                               |
// +-------------+---------------------------------+-------------+
// |             | [5. CENTER - TOP] Transport     |             |
// |             +---------------------------------+             |
// | [3. LEFT]   |                                 | [4. RIGHT]  |
// |  Track List | [6. CENTER - MAIN]              |  Attributes |
// |             |  Piano Roll / Grid              |             |
// |             |                                 |             |
// |-------------+---------------------------------+-------------+
// | [2. BOTTOM]                                                 |
// |  Velocity / Controller                                      |
// +-------------------------------------------------------------+

pub struct MidiApp {
    audio: Audio,
    shared_state: Arc<Mutex<SharedAudioState>>,
    open_file_name: String,
    start_octave: i8,
    midi_manager: Arc<Mutex<MidiManager>>,
    select_track: Option<u8>,
    solo_track: Option<u8>,
    show_keyboard: bool,
    show_track_list: bool,
    show_attributes: bool,
}

impl MidiApp {
    pub const APP_NAME: &str = "MIDI Editor";

    pub fn run() -> Result<(), Error> {
        // UI 실행
        let options = NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
            centered: true,
            persist_window: true,
            ..NativeOptions::default()
        };

        eframe::run_native(MidiApp::APP_NAME, options, Box::new(|cc| {
            // 폰트 설정
            let nanum_font = include_bytes!("../../assets/NanumGothic.ttf");
            let font_data = FontData::from_static(nanum_font);
            let font = FontInsert {
                name: "Nanum Gothic".to_string(),
                data: font_data,
                families: vec![InsertFontFamily {
                    family: FontFamily::Proportional,
                    priority: FontPriority::Highest,
                }],
            };
            cc.egui_ctx.add_font(font);

            // 오디오 설정
            let shared_state = Arc::new(Mutex::new(SharedAudioState::new()));
            let audio = Audio::new(shared_state.clone());

            let app = Box::new(MidiApp {
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
            });

            Ok(app)
        }))
    }

    fn open_file(&mut self) {
        // 파일 열기
        let path_buf_opt = FileDialog::default().add_filter("MIDI Files", &["mid", "midi"]).pick_file();
        if path_buf_opt.is_none() {
            get_message_box().lock().unwrap().error("No file selected.");
            return;
        }
        let path = path_buf_opt.unwrap();
        let file_path = path.as_os_str().to_str().unwrap().to_string();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        self.open_file_name = String::from(file_name);
        println!("file: {}", file_name);
        
        let midi_manager_arc = Arc::clone(&self.midi_manager);
        thread::spawn(move || {
            let mut midi_manager = midi_manager_arc.lock().unwrap();
            midi_manager.open(&file_path);
            get_message_box().lock().unwrap().show("File loaded successfully.");
        });
    }

    fn close(&mut self) {
        self.midi_manager.lock().unwrap().close();
        self.shared_state.lock().unwrap().is_playing = false;
        self.shared_state.lock().unwrap().playback_cursor = 0;
        self.open_file_name.clear();
    }

    fn play(&mut self) {
        let mut notes = Vec::new();
        let midi_manager = self.midi_manager.lock().unwrap();
        let sample_per_tick = (&midi_manager.meta.tempo * self.audio.sample_rate as f64) / (1_000_000f64 * &midi_manager.ppq);
        for (channel, midi) in &midi_manager.midi {
            let track = midi_manager.tracks.get(channel).unwrap();
            if !self.solo_track.is_none() && self.solo_track != Some(*channel) || track.is_muted { continue };
            for (key, data_list) in midi {
                for midi in data_list {
                    notes.push(MidiNote {
                        start_sample: (midi.tick * sample_per_tick as u64) as usize,
                        channel: *channel as i32,
                        key: *key as i32,
                        velocity: midi.velocity as i32,
                    });
                }
            }
        }

        let mut instruments = HashMap::new();
        for (channel, track) in &midi_manager.tracks {
            if !self.solo_track.is_none() && self.solo_track != Some(*channel) || track.is_muted { continue; }
            instruments.insert(*channel, track.instrument as i32);
        }

        let mut shared_state = self.shared_state.lock().unwrap();
        shared_state.notes = notes;
        shared_state.instruments = instruments;
    }
}

impl App for MidiApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // 아래 순서대로 패널을 그리지 않으면 위치가 의도와 달라지니 주의
        ctx.style_mut(|style| {
            style.interaction.selectable_labels = false;
        });
        let fill = Color32::from_rgb(0, 0, 0);
        let is_playing = self.shared_state.lock().unwrap().is_playing;
    
        let mut menu = Menu::default();
        let mut keyboard = Keyboard::default();
        let mut track_list = TrackList::default();
        let mut attributes = Attributes::default();
        let mut transport = Transport::default();
        let mut note_grid = NoteGrid::default();

        // TOP : 메뉴 바
        egui::TopBottomPanel::top(Menu::FRAME_NAME)
            .frame(egui::Frame::new().inner_margin(Menu::INNER_MARGIN).fill(fill))
            .exact_height(Menu::HEIGHT)
            .resizable(Menu::RESIZABLE)
            .show(ctx, |ui| menu.draw(ui, self));

        // BOTTOM : 키보드 건반
        if self.show_keyboard {
            egui::TopBottomPanel::bottom(Keyboard::FRAME_NAME)
                .frame(egui::Frame::new().inner_margin(Keyboard::INNER_MARGIN).fill(fill))
                .exact_height(Keyboard::HEIGHT)
                .resizable(Keyboard::RESIZABLE)
                .show(ctx, |ui| keyboard.draw(ui, self));
        }

        // LEFT : 트랙 리스트
        if self.show_track_list {
            egui::SidePanel::left(TrackList::FRAME_NAME)
                .frame(egui::Frame::new().inner_margin(TrackList::INNER_MARGIN).fill(fill))
                .exact_width(TrackList::WIDTH)
                .resizable(TrackList::RESIZABLE)
                .show(ctx, |ui| track_list.draw(ui, self));
        }

        // RIGHT : 선택된 노트/이벤트 속성 예시
        if self.show_attributes {
            egui::SidePanel::right(Attributes::FRAME_NAME)
                .frame(egui::Frame::new().inner_margin(Attributes::INNER_MARGIN).fill(fill))
                .exact_width(Attributes::WIDTH)
                .resizable(Attributes::RESIZABLE)
                .show(ctx, |ui| attributes.draw(ui, self));
        }

        // CENTER - TOP : 트랜스포트
        egui::TopBottomPanel::top(Transport::FRAME_NAME)
            .frame(egui::Frame::new().inner_margin(Transport::INNER_MARGIN).fill(fill))
            .exact_height(Transport::HEIGHT)
            .resizable(Transport::RESIZABLE)
            .show(ctx, |ui| transport.draw(ui, self));

        // CENTER - MAIN : 피아노 롤 / 그리드
        egui::CentralPanel::default()
            .frame(egui::Frame::new().inner_margin(NoteGrid::INNER_MARGIN).fill(fill))
            .show(ctx, |ui| {
                note_grid.draw(ui, self);

                // 메시지 박스 표시
                get_message_box().lock().unwrap().draw(ui);
            });

        // 재생 중일 때는 실시간으로 화면 업데이트
        if is_playing { ctx.request_repaint(); }
    }
}
