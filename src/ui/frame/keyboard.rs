use std::{cmp, collections::HashMap};

use egui::{Color32, Pos2, Stroke};

use crate::ui::{
    frame::{Frame, FRAME_HEAD_HEIGHT},
    MidiApp,
};

const WHITE_KEYS: &[u8] = &[0, 2, 4, 5, 7, 9, 11];
const WHITE_WIDTH: f32 = 24.0;
const BLACK_WIDTH: f32 = 14.0;
const BLACK_HEIGHT: f32 = 95.0;
const MAX_WHITE_KEYS: isize = 75;
const MIDDLE_POS: isize = 38;
const WHITE_KEY_TEXT_COLOR: Color32 = Color32::from_rgb(34, 34, 34);
const BLACK_KEY_TEXT_COLOR: Color32 = Color32::from_rgb(211, 211, 211);

#[derive(Default)]
pub struct Keyboard;

impl Frame for Keyboard {
    const FRAME_NAME: &str = "Keyboard";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 174.0;
    const RESIZABLE: bool = false;

    /// 현재 재생 커서에 맞춰 눌린 건반을 계산해 하단 키보드를 렌더링한다.
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        self.header(ui);

        let (is_playing, playback_cursor_samples) = {
            let shared_state = app.shared_state.lock().unwrap();
            (
                shared_state.is_playing,
                shared_state.playback_cursor_samples,
            )
        };

        let mut notes = HashMap::new();
        if is_playing {
            let midi_manager = app.midi_manager.lock().unwrap();
            if let Some(song) = midi_manager.song() {
                // 오디오와 같은 샘플 기준 커서를 다시 tick으로 환산해 활성 노트를 찾는다.
                let current_tick =
                    song.tick_for_sample(playback_cursor_samples, app.audio.sample_rate);
                for track in song.tracks.iter().filter(|track| match app.solo_track {
                    Some(track_index) => track.track_index == track_index && !track.is_muted,
                    None => !track.is_muted,
                }) {
                    for note in &track.note_spans {
                        if note.start_tick <= current_tick && current_tick < note.end_tick {
                            notes.insert(note.key, note.velocity);
                        }
                    }
                }
            }
        }

        let painter = ui.painter();
        let font_id = egui::FontId::new(10.0, egui::FontFamily::default());
        let min = ui.min_rect().min;
        let max = ui.max_rect().max;
        let width = max.x - min.x;
        let key_count = cmp::min(MAX_WHITE_KEYS, (width / WHITE_WIDTH).ceil() as isize);
        let start_x = min.x + (width - (key_count as f32 * WHITE_WIDTH)) / 2.0;
        let start_y = min.y + FRAME_HEAD_HEIGHT;
        let end_y = max.y;
        let start_key = cmp::max(0, MIDDLE_POS - (key_count as f32 / 2.0).ceil() as isize);
        let start_octave = app.start_octave + (start_key as i8) / 7;

        for i in 0..key_count {
            let note = (start_key + i) as i8;
            let octave = start_octave + note / 7;
            let real_note = octave as u8 * 12 + WHITE_KEYS[(note % 7) as usize];
            let is_pressed = notes.contains_key(&real_note);
            let x = start_x + i as f32 * WHITE_WIDTH;
            painter.rect(
                egui::Rect::from_two_pos(
                    Pos2::new(x, start_y),
                    Pos2::new(x + WHITE_WIDTH - 1.0, end_y - 1.0),
                ),
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    se: 2,
                    sw: 2,
                },
                if is_pressed {
                    Color32::YELLOW
                } else {
                    Color32::WHITE
                },
                Stroke::new(1.0, Color32::GRAY),
                egui::StrokeKind::Inside,
            );

            let name = (65 + (note + 2) as u8 % 7) as char;
            painter.text(
                Pos2::new(x + WHITE_WIDTH / 2.0, end_y - 2.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{name}{octave}"),
                font_id.clone(),
                WHITE_KEY_TEXT_COLOR,
            );
        }

        for i in 0..key_count {
            let note = (start_key + i) % 7;
            if matches!(note, 0 | 3) {
                continue;
            }

            let octave = start_octave + (start_key + i) as i8 / 7;
            let real_note = octave as u8 * 12 + WHITE_KEYS[(note % 7) as usize] - 1;
            let is_pressed = notes.contains_key(&real_note);
            let x = start_x + i as f32 * WHITE_WIDTH - (BLACK_WIDTH / 2.0);

            painter.rect(
                egui::Rect::from_two_pos(
                    Pos2::new(x, start_y),
                    Pos2::new(x + BLACK_WIDTH, start_y + BLACK_HEIGHT),
                ),
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    se: 1,
                    sw: 1,
                },
                if is_pressed {
                    Color32::YELLOW
                } else {
                    Color32::BLACK
                },
                Stroke::new(1.0, Color32::BLACK),
                egui::StrokeKind::Inside,
            );

            painter.rect(
                egui::Rect::from_two_pos(
                    Pos2::new(x + 2.0, start_y),
                    Pos2::new(x + BLACK_WIDTH - 2.0, start_y + BLACK_HEIGHT - 8.0),
                ),
                egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    se: 1,
                    sw: 1,
                },
                if is_pressed {
                    Color32::YELLOW
                } else {
                    Color32::BLACK
                },
                Stroke::new(1.0, Color32::from_rgb(64, 64, 64)),
                egui::StrokeKind::Inside,
            );

            let name = (65 + (note + 1) as u8 % 7) as char;
            painter.text(
                Pos2::new(x + BLACK_WIDTH / 2.0, start_y + BLACK_HEIGHT - 10.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{name}#"),
                font_id.clone(),
                BLACK_KEY_TEXT_COLOR,
            );
        }
    }
}
