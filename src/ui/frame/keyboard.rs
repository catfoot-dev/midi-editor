use std::{cmp, collections::HashMap};
use egui::{Color32, Pos2, Stroke};

use crate::ui::{MidiApp, frame::{FRAME_HEAD_HEIGHT, Frame}};

// 키보드 건반 예시
// +---------------------------------------------------------+
// | Keyboard                                                |
// +---------------------------------------------------------+
// | |=|=|||=|=|=|||=|=|||=|=|=|||=|=|||=|=|=|||=|=|||=|=|=| |
// |  | | | | | | | | | | | | | | | | | | | | | | | | | | |  |
// +---------------------------------------------------------+

// 실제 키
const WHITE_KEYS: &[u8] = &[
    0, 2, 4, 5, 7, 9, 11,
];

// 건반 크기 값
const WHITE_WIDTH: f32 = 24.0; // 흰 건반 길이: 150mm 내외, 검은 건반 앞으로 노출된 길이 48 ~ 52mm
const BLACK_WIDTH: f32 = 14.0; // 건반 폭: 국제 표준 23.5mm
const BLACK_HEIGHT: f32 = 95.0; // 검은 건반 길이: 95mm

// 최대 흰 건반 수
const MAX_WHITE_KEYS: isize = 75; // 전체 건반 수 88개 중 흰 건반 수는 75개

// 중간 건반(흰 건반)에 해당하는 상수 값
const MIDDLE_POS: isize = 38; // 중앙 건반 위치: 7(흰 건반 수) * 5(옥타브 수: -1 ~ 4) + 3(F 위치)

// 흰 건반 텍스트 색상
const WHITE_KEY_TEXT_COLOR: Color32 = Color32::from_rgb(34, 34, 34);
// 검은 건반 텍스트 색상
const BLACK_KEY_TEXT_COLOR: Color32 = Color32::from_rgb(211, 211, 211);

pub struct Keyboard {
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
        }
    }
}

impl Frame for Keyboard {
    const FRAME_NAME: &str = "Keyboard";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 174.0;
    const RESIZABLE: bool = false;

    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        // BOTTOM : 키보드 건반
        self.header(ui);

        // 현재 눌러져 있는 건반 체크
        let mut notes = HashMap::new();
        if app.shared_state.lock().unwrap().is_playing {
            let sample_rate = app.audio.sample_rate as f64;
            let playback_cursor = app.shared_state.lock().unwrap().playback_cursor as u64;
            let current_seconds = playback_cursor as f64 / sample_rate;
            let beats_per_second = {
                let midi_manager = &app.midi_manager.lock().unwrap();
                1_000_000f64 / midi_manager.meta.tempo
            };
            let ticks_per_second = {
                let midi_manager = &app.midi_manager.lock().unwrap();
                beats_per_second * midi_manager.ppq
            };
            for (_, midi) in &app.midi_manager.lock().unwrap().midi {
                for (key, data_list) in midi {
                    for data in data_list {
                        let seconds = data.tick as f64 / ticks_per_second;
                        if seconds > current_seconds { continue; }
                        if data.is_on && data.velocity > 0 {
                            notes.insert(*key, data.velocity);
                        } else {
                            notes.remove(key);
                        }
                    }
                }
            }
        }

        let painter = ui.painter();
        let font_id = egui::FontId::new(10.0, egui::FontFamily::default());

        // 그릴 수 있는 영역
        let min = ui.min_rect().min;
        let max = ui.max_rect().max;
        let width = max.x - min.x;

        // 그릴 흰 건반 수
        let key_count = cmp::min(MAX_WHITE_KEYS, (width / WHITE_WIDTH).ceil()as isize);

        // 그릴 좌표 계산
        let start_x = min.x + (width - (key_count as f32 * WHITE_WIDTH)) / 2.0;
        let start_y = min.y + FRAME_HEAD_HEIGHT;
        let end_y = max.y;

        // 첫번째 음의 건반, 옥타브 계산
        let start_key = cmp::max(0, MIDDLE_POS - (key_count as f32 / 2.0).ceil() as isize);
        let start_octave = app.start_octave + (start_key as i8) / 7;

        // 흰 건반 영역
        for i in 0..key_count {
            let note = (start_key + i) as i8;
            let octave = start_octave + note / 7;
            let real_note = octave as u8 * 12 + WHITE_KEYS[(note % 7) as usize];
            let is_pressed = notes.contains_key(&real_note);

            // 흰 건반 그리기
            let x = start_x + i as f32 * WHITE_WIDTH;
            painter.rect(
                egui::Rect::from_two_pos(
                    Pos2::new(x, start_y),
                    Pos2::new(x + WHITE_WIDTH - 1.0, end_y - 1.0),
                ),
                egui::CornerRadius { nw: 0, ne: 0, se: 2, sw: 2  },
                if is_pressed { Color32::YELLOW } else { Color32::WHITE },
                Stroke::new(1.0, Color32::GRAY),
                egui::StrokeKind::Inside,
            );
            
            // 건반 위에 음 이름 표시
            let name = (65 + (note + 2) as u8 % 7) as char;
            painter.text(
                Pos2::new(x + WHITE_WIDTH / 2.0, end_y - 2.0),
                egui::Align2::CENTER_BOTTOM,
                format!("{name}{octave}"),
                font_id.clone(),
                WHITE_KEY_TEXT_COLOR,
            );
        }

        // 검은 건반 영역
        for i in 0..key_count {
            let note = (start_key + i) % 7;
            if matches!(note, 0 | 3) { continue; }
            let octave = start_octave + (start_key + i) as i8 / 7;
            let real_note = octave as u8 * 12 + WHITE_KEYS[(note % 7) as usize] - 1;
            let is_pressed = notes.contains_key(&real_note);
        
            // 검은 건반 그리기
            let x = start_x + i as f32 * WHITE_WIDTH - (BLACK_WIDTH / 2.0);
            painter.rect(
                egui::Rect::from_two_pos(
                    Pos2::new(x, start_y),
                    Pos2::new(x + BLACK_WIDTH, start_y + BLACK_HEIGHT),
                ),
                egui::CornerRadius { nw: 0, ne: 0, se: 1, sw: 1  },
                if is_pressed { Color32::YELLOW } else { Color32::BLACK },
                Stroke::new(1.0, Color32::BLACK),
                egui::StrokeKind::Inside,
            );

            // 검은 건반 입체 효과
            painter.rect(
                egui::Rect::from_two_pos(
                    Pos2::new(x + 2.0, start_y),
                    Pos2::new(x + BLACK_WIDTH - 2.0, start_y + BLACK_HEIGHT - 8.0),
                ),
                egui::CornerRadius { nw: 0, ne: 0, se: 1, sw: 1  },
                if is_pressed { Color32::YELLOW } else { Color32::BLACK },
                Stroke::new(1.0, Color32::from_rgb(64, 64, 64)),
                egui::StrokeKind::Inside,
            );

            // 건반 위에 음 이름 표시
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
