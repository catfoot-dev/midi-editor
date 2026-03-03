use egui::{
    Color32, Margin, Slider, SliderOrientation
};

use crate::ui::{MidiApp, frame::Frame};

// 트랜스포트 (MIDI 재생 제어) 예시
// +-------------------------------------------------------------+
// | midi_file.mid                                               |
// | [⏮] [⏸] [⏹] [🔁]   ------O---------                        |
// +-------------------------------------------------------------+

pub struct Transport {
}

impl Default for Transport {
    fn default() -> Self {
        Self {
        }
    }
}

impl Frame for Transport {
    const FRAME_NAME: &str = "Transport";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 52.0;
    const RESIZABLE: bool = false;

    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        let is_file_open = !app.open_file_name.is_empty();

        egui::Frame::new()
            .inner_margin(Margin::same(2))
            .fill(Color32::from_rgb(32, 32, 32))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width());

                        let title = if !is_file_open {
                            // 파일이 열려있지 않을 때 MIDI 파일을 선택해달라는 메시지 표시
                            "Select a MIDI file to play.".to_string()
                        } else {
                            // 파일이 열려있을 때 파일 이름 표시
                            let program_name = app.midi_manager.lock().unwrap().meta.program_name.clone();
                            if program_name.is_empty() { app.open_file_name.clone() } else { program_name }
                        };
                        let title_label = egui::Label::new(title)
                            .wrap_mode(egui::TextWrapMode::Truncate);
                        ui.add_sized([ui.available_width(), 22.0], title_label);

                        // 컨트롤러 버튼
                        ui.horizontal_centered(|ui| {
                            ui.set_min_width(ui.available_width());

                            // 파일 열기/닫기
                            let is_empty_file = app.open_file_name.is_empty();
                            let open_btn = egui::Button::new(if is_empty_file { "Open" } else { "Close" });
                            let open_btn_ui = ui.add(open_btn);
                            if open_btn_ui.clicked() {
                                if is_empty_file {
                                    app.open_file();
                                } else {
                                    app.close();
                                }
                            }

                            // 처음으로
                            let rewind_btn = egui::Button::new("⏮");
                            let is_not_begin = is_file_open && app.shared_state.lock().unwrap().playback_cursor != 0;
                            let rewind_btn_ui = ui.add_enabled(is_not_begin, rewind_btn)
                                .on_hover_text("Rewind");
                            if rewind_btn_ui.clicked() {
                                app.shared_state.lock().unwrap().playback_cursor = 0;
                            }

                            // 재생/일시정지
                            let is_playing = is_file_open && app.shared_state.lock().unwrap().is_playing;
                            let play_pause_text = if is_playing { "⏸" } else { "▶" };
                            let play_pause_btn = egui::Button::new(play_pause_text);
                            let play_pause_btn_ui = ui.add_enabled(is_file_open, play_pause_btn)
                                .on_hover_text("Play/Pause");
                            if play_pause_btn_ui.clicked() {
                                let is_playing = !app.shared_state.lock().unwrap().is_playing;
                                app.shared_state.lock().unwrap().is_playing = is_playing;
                                if is_playing { app.play(); }
                            }

                            // 정지
                            let stop_btn = egui::Button::new("⏹");
                            let stop_btn_ui = ui.add_enabled(is_playing, stop_btn)
                                .on_hover_text("Stop");
                            if stop_btn_ui.clicked() {
                                app.shared_state.lock().unwrap().is_playing = false;
                                app.shared_state.lock().unwrap().playback_cursor = 0;
                            }

                            // 반복
                            let repeat_text = if app.shared_state.lock().unwrap().is_repeat { "🔁" } else { "❶" };
                            let repeat_btn = egui::Button::new(repeat_text);
                            let repeat_btn_ui = ui.add_enabled(is_file_open, repeat_btn)
                                .on_hover_text("Once/Repeat");
                            if repeat_btn_ui.clicked() {
                                app.shared_state.lock().unwrap().is_repeat = !app.shared_state.lock().unwrap().is_repeat;
                            }

                            // 재생 슬라이더
                            let sample_rate = app.audio.sample_rate as f64;
                            let current_seconds = app.shared_state.lock().unwrap().playback_cursor as f64 / sample_rate;
                            let total_seconds = app.midi_manager.lock().unwrap().total_seconds;
                            let mut value = (current_seconds / total_seconds * 1000.0) as i32;
                            let control_slider = Slider::new(&mut value, 0..=1000)
                                .orientation(SliderOrientation::Horizontal)
                                .handle_shape(egui::style::HandleShape::Circle)
                                .step_by(1.0)
                                .trailing_fill(true)
                                .show_value(false);
                            ui.spacing_mut().slider_width = 200.0;
                            let control_slider_ui = ui.add_enabled(is_file_open, control_slider);
                            if control_slider_ui.drag_stopped() {
                                let playback_cursor = (value as f64 / 1000.0 * total_seconds) as usize;
                                app.shared_state.lock().unwrap().playback_cursor = playback_cursor;
                                println!("playback_cursor: {}, value: {}", playback_cursor, value);
                            }
                        });
                    });
                });
            });
    }
}
