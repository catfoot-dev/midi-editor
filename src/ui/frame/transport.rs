use egui::{Color32, Margin, Slider, SliderOrientation};

use crate::ui::{MidiApp, frame::Frame};

#[derive(Default)]
pub struct Transport;

impl Frame for Transport {
    const FRAME_NAME: &str = "Transport";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 52.0;
    const RESIZABLE: bool = false;

    /// 재생 제어 버튼과 탐색 슬라이더를 그리고 SharedAudioState를 직접 갱신한다.
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        let midi_manager = app.midi_manager.lock().unwrap();
        let is_loading = midi_manager.is_loading();
        let song = midi_manager.song();
        let title = if is_loading {
            "Loading MIDI file...".to_string()
        } else if let Some(song) = song {
            song.display_title()
                .map(str::to_string)
                .unwrap_or_else(|| app.open_file_name.clone())
        } else {
            "Select a MIDI file to play.".to_string()
        };
        drop(midi_manager);

        let (is_playing, repeat_enabled, playback_cursor_samples, song_length_samples) = {
            let shared_state = app.shared_state.lock().unwrap();
            (
                shared_state.is_playing,
                shared_state.repeat_enabled,
                shared_state.playback_cursor_samples,
                shared_state.song_length_samples,
            )
        };
        let is_file_open = !app.open_file_name.is_empty();
        // 슬라이더는 항상 0..1000 범위로 유지하고 실제 샘플 위치로만 환산한다.
        let slider_value = if song_length_samples == 0 {
            0
        } else {
            ((playback_cursor_samples as f64 / song_length_samples as f64) * 1000.0) as i32
        };

        egui::Frame::new()
            .inner_margin(Margin::same(2))
            .fill(Color32::from_rgb(32, 32, 32))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(ui.available_width());

                        let title_label =
                            egui::Label::new(title).wrap_mode(egui::TextWrapMode::Truncate);
                        ui.add_sized([ui.available_width(), 22.0], title_label);

                        ui.horizontal_centered(|ui| {
                            ui.set_min_width(ui.available_width());

                            let open_text = if is_loading {
                                "Loading..."
                            } else if is_file_open {
                                "Close"
                            } else {
                                "Open"
                            };
                            let open_btn = egui::Button::new(open_text);
                            if ui.add_enabled(!is_loading, open_btn).clicked() {
                                if is_file_open {
                                    app.close();
                                } else {
                                    app.open_file();
                                }
                            }

                            let rewind_btn = egui::Button::new("⏮");
                            let can_rewind = is_file_open && playback_cursor_samples != 0;
                            if ui
                                .add_enabled(can_rewind, rewind_btn)
                                .on_hover_text("Rewind")
                                .clicked()
                            {
                                app.shared_state.lock().unwrap().seek_samples(0);
                            }

                            let play_pause_text = if is_playing { "⏸" } else { "▶" };
                            let play_pause_btn = egui::Button::new(play_pause_text);
                            if ui
                                .add_enabled(is_file_open && !is_loading, play_pause_btn)
                                .on_hover_text("Play/Pause")
                                .clicked()
                            {
                                if is_playing {
                                    app.shared_state.lock().unwrap().set_playing(false);
                                } else {
                                    app.play();
                                }
                            }

                            let stop_btn = egui::Button::new("⏹");
                            if ui
                                .add_enabled(is_playing, stop_btn)
                                .on_hover_text("Stop")
                                .clicked()
                            {
                                app.stop();
                            }

                            let repeat_btn =
                                egui::Button::new(if repeat_enabled { "🔁" } else { "❶" });
                            if ui
                                .add_enabled(is_file_open, repeat_btn)
                                .on_hover_text("Once/Repeat")
                                .clicked()
                            {
                                let enabled = !repeat_enabled;
                                app.shared_state.lock().unwrap().set_repeat_enabled(enabled);
                            }

                            let mut value = slider_value;
                            let control_slider = Slider::new(&mut value, 0..=1000)
                                .orientation(SliderOrientation::Horizontal)
                                .handle_shape(egui::style::HandleShape::Circle)
                                .step_by(1.0)
                                .trailing_fill(true)
                                .show_value(false);
                            ui.spacing_mut().slider_width = 200.0;
                            let response = ui.add_enabled(is_file_open, control_slider);
                            if response.drag_stopped() && song_length_samples > 0 {
                                let sample_index =
                                    ((value as f64 / 1000.0) * song_length_samples as f64).round()
                                        as usize;
                                app.shared_state.lock().unwrap().seek_samples(sample_index);
                            }
                        });
                    });
                });
            });
    }
}
