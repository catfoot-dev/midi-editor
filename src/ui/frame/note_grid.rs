use egui::{Color32, Pos2, Stroke};

use crate::ui::{MidiApp, frame::Frame};

const NOTE_NAMES: &[&str] = &[
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

const CHANNEL_COLORS: [Color32; 16] = [
    Color32::from_rgb(197, 131, 56),
    Color32::from_rgb(19, 178, 210),
    Color32::from_rgb(232, 198, 29),
    Color32::from_rgb(198, 200, 54),
    Color32::from_rgb(208, 96, 145),
    Color32::from_rgb(71, 165, 114),
    Color32::from_rgb(229, 108, 77),
    Color32::from_rgb(99, 128, 222),
    Color32::from_rgb(209, 154, 102),
    Color32::from_rgb(134, 167, 60),
    Color32::from_rgb(61, 185, 154),
    Color32::from_rgb(211, 102, 123),
    Color32::from_rgb(162, 121, 255),
    Color32::from_rgb(94, 201, 255),
    Color32::from_rgb(255, 170, 70),
    Color32::from_rgb(140, 140, 140),
];

#[derive(Default)]
pub struct NoteGrid;

impl Frame for NoteGrid {
    const FRAME_NAME: &str = "NoteGrid";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 0.0;
    const RESIZABLE: bool = true;

    /// note span과 현재 재생 커서를 이용해 피아노 롤과 타임라인을 그린다.
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        let rect = ui.response().rect;
        let label_width = 35.0;
        let beats_width = 50.0;
        let row_height = 12.0;
        let beats_height = 20.0;
        let max_height = 128.0 * row_height + beats_height;

        let midi_manager = app.midi_manager.lock().unwrap();
        let Some(song) = midi_manager.song() else {
            ui.centered_and_justified(|ui| {
                ui.label("Open a MIDI file to see the piano roll.");
            });
            return;
        };

        // 그리드는 tempo 변화와 무관하게 tick/beat 축으로 그려야 편집 좌표가 안정적이다.
        let tick_width = if song.ppq == 0 {
            0.0
        } else {
            beats_width / song.ppq as f32
        };
        let max_width =
            label_width + song.beats_for_ticks(song.total_ticks) * beats_width + beats_width;
        let max_width = max_width.max(ui.available_width());

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

                let start_x = response.min.x;
                let start_y = response.min.y;
                let grid_start_x = start_x + label_width;
                let grid_start_y = start_y + beats_height;
                let width = response.max.x;

                for i in 0..=127 {
                    let note = 127 - i;
                    let color = match note % 12 {
                        1 | 3 | 6 | 8 | 10 => Color32::GRAY,
                        _ => Color32::LIGHT_GRAY,
                    };
                    let y = grid_start_y + (i * 12) as f32;
                    painter.line_segment(
                        [Pos2::new(grid_start_x, y + 6.0), Pos2::new(width, y + 6.0)],
                        Stroke::new(11.0, color),
                    );
                }

                for track in song.tracks.iter().filter(|track| match app.solo_track {
                    Some(track_index) => track.track_index == track_index && !track.is_muted,
                    None => !track.is_muted,
                }) {
                    let color = CHANNEL_COLORS[track.channel as usize % CHANNEL_COLORS.len()];
                    for note in &track.note_spans {
                        let x = grid_start_x + note.start_tick as f32 * tick_width;
                        let end_x = grid_start_x + note.end_tick as f32 * tick_width;
                        let y = grid_start_y + (127 - note.key) as f32 * row_height;
                        painter.rect(
                            egui::Rect::from_points(&[
                                Pos2::new(x, y + 1.0),
                                Pos2::new(end_x.max(x + 1.0), y + row_height),
                            ]),
                            1.5,
                            color,
                            Stroke::new(1.0, Color32::WHITE),
                            egui::StrokeKind::Inside,
                        );
                    }
                }

                for i in 0..=127 {
                    let note = 127 - i;
                    let color = match note % 12 {
                        1 | 3 | 6 | 8 | 10 => Color32::GRAY,
                        _ => Color32::LIGHT_GRAY,
                    };
                    let y = grid_start_y + (i * 12) as f32;
                    let rgb = color.r().saturating_sub(50);
                    painter.rect_filled(
                        egui::Rect::from_points(&[
                            Pos2::new(rect.min.x, y + 1.0),
                            Pos2::new(rect.min.x + label_width - 1.0, y + row_height),
                        ]),
                        0.0,
                        Color32::from_rgb(rgb, rgb, rgb),
                    );
                    painter.text(
                        Pos2::new(rect.min.x + 3.0, y),
                        egui::Align2::LEFT_TOP,
                        format!("{}{}", NOTE_NAMES[note % 12], (note / 12) as i8 - 1),
                        font_id.clone(),
                        Color32::from_rgb(34, 34, 34),
                    );
                }

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

                for i in 0..=(max_width / beats_width) as usize {
                    let x = grid_start_x + i as f32 * beats_width;
                    if x < rect.min.x + label_width || x > rect.max.x {
                        continue;
                    }

                    if i > 0 {
                        painter.line_segment(
                            [Pos2::new(x, rect.min.y), Pos2::new(x, rect.max.y)],
                            Stroke::new(1.0, Color32::DARK_GRAY),
                        );
                    }

                    for j in 1..8 {
                        let pos_x = x + j as f32 * beats_width / 8.0;
                        let line_height = if j % 2 == 1 { 1.5 } else { 2.2 };
                        painter.line_segment(
                            [
                                Pos2::new(pos_x, rect.min.y + beats_height / line_height),
                                Pos2::new(pos_x, rect.min.y + beats_height),
                            ],
                            Stroke::new(1.0, Color32::DARK_GRAY),
                        );
                    }

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
                        Pos2::new(rect.min.x + label_width - 1.0, rect.max.y),
                    ],
                    Stroke::new(1.0, Color32::BLACK),
                );

                let current_sample = app.shared_state.lock().unwrap().playback_cursor_samples;
                let current_tick = song.tick_for_sample(current_sample, app.audio.sample_rate);
                let timeline_x = grid_start_x + song.beats_for_ticks(current_tick) * beats_width;
                if timeline_x >= rect.min.x + label_width && timeline_x <= rect.max.x {
                    painter.line_segment(
                        [
                            Pos2::new(timeline_x, rect.min.y),
                            Pos2::new(timeline_x, rect.max.y),
                        ],
                        Stroke::new(1.0, Color32::from_rgb(62, 46, 211)),
                    );
                }
            });
    }
}
