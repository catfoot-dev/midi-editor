use egui::{
    Color32,
    Pos2,
    Stroke,
};

use crate::ui::{MidiApp, frame::Frame};

// 피아노 롤 / 그리드 예시
// +-----+-------+-------+-------+-------+-------+-------+-------+
// | G9  |       | [===] |[===]  |       |       |       |       |
// | F#9 |       |       |       |       |       |       |       |
// | F9  |       |       |     [===========]     |       |       |
// | E9  |       |    [=====]    |       |   [==]|       |       |
// | D#9 |       |       |       |       |       |       |       |
// |                              ...                            |
// | D3  |       |       |       | [===] |       |       |       |
// +-----+-------+-------+-------+-------+-------+-------+-------+

const NOTE_NAMES: &[&'static str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
];

const CHANNEL_COLORS: &[Color32] = &[
    Color32::from_rgb(197, 131, 56),
    Color32::from_rgb(19, 178, 210),
    Color32::from_rgb(232, 198, 29),
    Color32::from_rgb(198, 200, 54),
];

pub struct NoteGrid {
}

impl Default for NoteGrid {
    fn default() -> Self {
        Self {
        }
    }
}

impl Frame for NoteGrid {
    const FRAME_NAME: &str = "NoteGrid";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 0.0;
    const RESIZABLE: bool = true;

    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        let rect = ui.response().rect;
        let label_width = 35.0; // 음 이름 표시 너비 값
        let beats_width = 50.0; // 1박자당 너비 값
        let row_height = 12.0;
        let beats_height = 20.0;
        let max_height = 128.0 * row_height + beats_height; // 128음역 * 음 높이(12픽셀)
        let tick_width = {
            let midi_manager = &app.midi_manager.lock().unwrap();
            beats_width / midi_manager.ppq as f32 // 1틱장 너비 값
        };
        let beats_per_second = {
            let midi_manager = &app.midi_manager.lock().unwrap();
            1_000_000f64 / midi_manager.meta.tempo
        };
        // let ticks_per_second = {
        //     let midi_manager = &app.midi_manager.lock().unwrap();
        //     beats_per_second * midi_manager.ppq
        // };
        let max_width = {
            let midi_manager = &app.midi_manager.lock().unwrap();
            let width = label_width + (midi_manager.total_seconds * beats_per_second) as f32 * beats_width + beats_width;
            if width > ui.available_width() { width } else { ui.available_width() }
        };

        // 노트 그리드 영역
        egui::ScrollArea::both()
            .max_height(max_height)
            .hscroll(true)
            .vscroll(true)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let size = egui::vec2(max_width, max_height);
                let (response, _rect) = ui.allocate_exact_size(size, egui::Sense::hover());
                let painter = ui.painter().with_clip_rect(ui.clip_rect());
                let font_id = egui::FontId::new(10.0, egui::FontFamily::default());

                // 기준 좌표 계산
                let start_x = response.min.x;
                let start_y = response.min.y;
                let grid_start_x = start_x + label_width;
                let grid_start_y = start_y + beats_height;
                let width = response.max.x;
                // let height = response.max.y;

                for i in 0..=127 {
                    let note = 127 - i;
                    let color = match note % 12 {
                        1 | 3 | 6 | 8 | 10 => Color32::GRAY,
                        _ => Color32::LIGHT_GRAY,
                    };
                    let y = grid_start_y + (i * 12) as f32;

                    // 노트 영역 배경색
                    let start_point = Pos2::new(grid_start_x, y + 6.0);
                    let end_point = Pos2::new(width, y + 6.0);
                    painter.line_segment(
                        [start_point, end_point],
                        Stroke::new(11.0, color),
                    );
                }

                // 키 노트 그리기
                for (channel, midi) in &app.midi_manager.lock().unwrap().midi {
                    for (key, data_list) in midi {
                        let mut start = 0u64;
                        let mut end = 0u64;
                        for data in data_list {
                            if data.is_on {
                                if start == 0u64 {
                                    start = data.tick;
                                } else if data.velocity == 0u8 {
                                    end = data.tick;
                                }
                            } else {
                                end = data.tick;
                            }

                            if end != 0u64 {
                                let x = grid_start_x + start as f32 * tick_width;
                                let end_x = grid_start_x + end as f32 * tick_width;
                                let y = grid_start_y + (127 - *key) as f32 * row_height;
                                start = 0;
                                end = 0;
                                painter.rect(
                                    egui::Rect::from_points(&[
                                        Pos2::new(x, y + 1.0),
                                        Pos2::new(end_x, y + row_height)
                                    ]),
                                    1.5,
                                    CHANNEL_COLORS[*channel as usize],
                                    Stroke::new(1.0, Color32::WHITE),
                                    egui::StrokeKind::Inside,
                                );
                            }
                        }
                    }
                }

                // 음 이름 표시
                for i in 0..=127 {
                    let note = 127 - i;
                    let color = match note % 12 {
                        1 | 3 | 6 | 8 | 10 => Color32::GRAY,
                        _ => Color32::LIGHT_GRAY,
                    };
                    let y = grid_start_y + (i * 12) as f32;

                    // 음 이름 표시 영역 배경색
                    let rgb = color.r() - 50;
                    painter.rect_filled(
                        egui::Rect::from_points(&[
                            Pos2::new(rect.min.x, y + 1.0),
                            Pos2::new(rect.min.x + label_width - 1.0, y + row_height)
                        ]),
                        0.0, Color32::from_rgb(rgb, rgb, rgb),
                    );

                    // 음 이름 텍스트 표시
                    painter.text(
                        Pos2::new(rect.min.x + 3.0, y),
                        egui::Align2::LEFT_TOP,
                        format!("{}{}", NOTE_NAMES[note % 12], (note / 12) as i8 - 1),
                        font_id.clone(),
                        Color32::from_rgb(34, 34, 34),
                    );
                }

                // 타임라인 영역
                painter.rect(
                    egui::Rect::from_two_pos(
                        Pos2::new(rect.min.x, rect.min.y),
                        Pos2::new(rect.max.x, rect.min.y + beats_height),
                    ),
                    0.0,
                    Color32::from_rgb(32, 32, 32),
                    Stroke::new(1.0, Color32::BLACK),
                    egui::StrokeKind::Outside,
                );

                // 박자 구분을 위한 가이드 라인
                for i in 0..=(max_width/beats_width) as usize {
                    let x = grid_start_x + i as f32 * beats_width;
                    // 화면 표시 영역을 벗어날 경우 그리지 않음
                    if x < rect.min.x + label_width || x > rect.max.x { continue };
                    if i > 0 {
                        // 1박자 선
                        let start_point = Pos2::new(x, rect.min.y);
                        let end_point = Pos2::new(x, rect.max.y);
                        painter.line_segment(
                            [start_point, end_point],
                            Stroke::new(1.0, Color32::DARK_GRAY),
                        );
                    }

                    // 1박자를 10분할한 선
                    for j in 1..8 {
                        let pos_x = x + j as f32 * beats_width / 8.0;
                        let line_height = if j % 2 == 1 { 1.5 } else { 2.2 };
                        let start_point = Pos2::new(pos_x, rect.min.y + beats_height / line_height);
                        let end_point = Pos2::new(pos_x, rect.min.y + beats_height);
                        painter.line_segment(
                            [start_point, end_point],
                            Stroke::new(1.0, Color32::DARK_GRAY),
                        );
                    }

                    // 박자 텍스트 표시
                    painter.text(
                        Pos2::new(x + 1.0, rect.min.y),
                        egui::Align2::LEFT_TOP,
                        format!("{}", i + 1),
                        font_id.clone(),
                        Color32::WHITE,
                    );
                }
                painter.line_segment(
                    [
                        Pos2::new(rect.min.x + label_width - 1.0, rect.min.y),
                        Pos2::new(rect.min.x + label_width - 1.0, rect.max.y)
                    ],
                    Stroke::new(1.0, Color32::BLACK),
                );

                // 타임라인 선
                let timeline_x = {
                    let sample_rate = app.audio.sample_rate as f64;
                    let current_seconds = app.shared_state.lock().unwrap().playback_cursor as f64 / sample_rate;
                    if current_seconds == 0.0 { 0.0 } else {
                        grid_start_x + (current_seconds * beats_per_second) as f32 * beats_width
                    }
                };
                if timeline_x >= rect.min.x + label_width && timeline_x <= rect.max.x {
                    painter.line_segment(
                        [
                            Pos2::new(timeline_x, rect.min.y),
                            Pos2::new(timeline_x, rect.max.y)
                        ],
                        Stroke::new(1.0, Color32::from_rgb(62, 46, 211)),
                    );
                }
            });
    }
}
