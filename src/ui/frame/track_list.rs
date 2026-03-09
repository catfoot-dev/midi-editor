use egui::Color32;

use crate::{
    midi::instruments::{DRUM_KITS, MIDI_GROUPS, MIDI_INSTRUMENTS},
    ui::{MidiApp, frame::Frame},
};

#[derive(Default)]
pub struct TrackList;

impl Frame for TrackList {
    const FRAME_NAME: &str = "TrackList";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 225.0;
    const HEIGHT: f32 = 0.0;
    const RESIZABLE: bool = false;

    /// 트랙 목록과 확장 편집 UI를 테이블 형태로 렌더링한다.
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        use egui_extras::{Column, TableBuilder};

        self.header(ui);

        let available_height = ui.available_height();
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(210.0))
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    let text_height = ui.text_style_height(&egui::TextStyle::Body);
                    ui.horizontal(|ui| {
                        ui.add_sized([128.0, text_height], egui::Label::new("Track"));
                        ui.add_sized([18.0, text_height], egui::Label::new("CH"));
                        ui.add_sized([17.0, text_height], egui::Label::new("S"));
                        ui.add_sized([18.0, text_height], egui::Label::new("M"));
                    });
                });
            })
            .body(|mut body| self.set_track(&mut body, app));
    }
}

impl TrackList {
    /// 트랙별 솔로/뮤트/채널/악기 편집을 반영하고 필요하면 재생 타임라인을 갱신한다.
    fn set_track(&mut self, body: &mut egui_extras::TableBody<'_>, app: &mut MidiApp) {
        let mut playback_dirty = false;

        {
            let mut midi_manager = app.midi_manager.lock().unwrap();
            let Some(song) = midi_manager.song_mut() else {
                return;
            };

            for track in &mut song.tracks {
                let is_extended = app.select_track == Some(track.track_index);
                let row_height = if is_extended { 96.0 } else { 20.0 };

                body.row(row_height, |mut row| {
                    row.col(|ui| {
                        let label_width = 52.0;
                        let line_height = ui.text_style_height(&egui::TextStyle::Body);

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let accordion_text = if is_extended { "-" } else { "+" };
                                let accordion_btn = egui::Button::new(accordion_text).small();
                                if ui
                                    .add_sized([20.0, line_height], accordion_btn)
                                    .on_hover_text("Details")
                                    .clicked()
                                {
                                    app.select_track = if is_extended {
                                        None
                                    } else {
                                        Some(track.track_index)
                                    };
                                }

                                let track_label = egui::Label::new(track.display_name().to_string());
                                ui.add_sized([100.0, line_height], track_label);

                                let channel_label =
                                    egui::Label::new(format!("{:02}", track.channel + 1));
                                ui.add_sized([20.0, line_height], channel_label);

                                let mut is_solo = app.solo_track == Some(track.track_index);
                                if ui.checkbox(&mut is_solo, "").clicked() {
                                    app.solo_track = if is_solo {
                                        Some(track.track_index)
                                    } else {
                                        None
                                    };
                                    playback_dirty = true;
                                }

                                if ui.checkbox(&mut track.is_muted, "").changed() {
                                    playback_dirty = true;
                                }
                            });

                            if !is_extended {
                                return;
                            }

                            ui.separator();

                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::LEFT),
                                |ui| {
                                    ui.add_sized([label_width, line_height], egui::Label::new("Channel:"));
                                    // 같은 위젯이 여러 행에 생기므로 트랙 인덱스를 포함한 고유 ID를 준다.
                                    egui::ComboBox::from_id_salt(("track_channel", track.track_index))
                                        .width(40.0)
                                        .selected_text(format!("{:02}", track.channel + 1))
                                        .show_ui(ui, |ui| {
                                            for channel in 0_u8..16 {
                                                if ui
                                                    .selectable_value(
                                                        &mut track.channel,
                                                        channel,
                                                        format!("{:02}", channel + 1),
                                                    )
                                                    .changed()
                                                {
                                                    playback_dirty = true;
                                                }
                                            }
                                        });
                                },
                            );

                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::LEFT),
                                |ui| {
                                    ui.add_sized(
                                        [label_width, line_height],
                                        egui::Label::new("Instrument:"),
                                    );
                                    let selected_text =
                                        current_instrument_name(track.channel, track.program);

                                    // 악기 선택도 행마다 별도 상태를 가져야 하므로 고유 ID를 사용한다.
                                    egui::ComboBox::from_id_salt(("track_program", track.track_index))
                                        .width(140.0)
                                        .selected_text(selected_text)
                                        .show_ui(ui, |ui| {
                                            if track.is_percussion() {
                                                let mut kits: Vec<(u8, &'static str)> =
                                                    DRUM_KITS.entries().map(|(k, v)| (*k, *v)).collect();
                                                kits.sort_by_key(|(program, _)| *program);
                                                for (program, name) in kits {
                                                    if ui
                                                        .selectable_value(
                                                            &mut track.program,
                                                            program,
                                                            format!("{program}. {name}"),
                                                        )
                                                        .changed()
                                                    {
                                                        track.instrument_name = name.to_string();
                                                        track.program_changes = vec![crate::audio::midi_struct::ProgramChangeEvent {
                                                            tick: 0,
                                                            program,
                                                        }];
                                                        playback_dirty = true;
                                                    }
                                                }
                                            } else {
                                                for (program, instrument) in
                                                    MIDI_INSTRUMENTS.iter().enumerate()
                                                {
                                                    let group =
                                                        &MIDI_GROUPS[instrument.group as usize];
                                                    if ui
                                                        .selectable_value(
                                                            &mut track.program,
                                                            program as u8,
                                                            format!(
                                                                "{}. [{}] {}",
                                                                program, group.name, instrument.name
                                                            ),
                                                        )
                                                        .changed()
                                                    {
                                                        track.instrument_name =
                                                            instrument.name.to_string();
                                                        track.program_changes = vec![crate::audio::midi_struct::ProgramChangeEvent {
                                                            tick: 0,
                                                            program: program as u8,
                                                        }];
                                                        playback_dirty = true;
                                                    }
                                                }
                                            }
                                        });
                                },
                            );

                            ui.with_layout(
                                egui::Layout::left_to_right(egui::Align::LEFT),
                                |ui| {
                                    ui.add_sized([label_width, line_height], egui::Label::new("Volume:"));
                                    ui.horizontal(|ui| {
                                        let slider = egui::Slider::new(&mut track.volume, 0.0..=100.0)
                                            .show_value(false);
                                        ui.add_enabled(!track.is_muted, slider);

                                        let font_id =
                                            egui::FontId::new(12.0, egui::FontFamily::default());
                                        let volume_text = format!("{}%", track.volume as u8);
                                        let volume_pos = ui.min_rect().center();
                                        ui.painter().text(
                                            volume_pos,
                                            egui::Align2::CENTER_CENTER,
                                            volume_text,
                                            font_id,
                                            Color32::GRAY,
                                        );
                                    });
                                },
                            );

                            ui.separator();
                        });
                    });
                });
            }
        }

        if playback_dirty {
            app.refresh_playback_data(false);
        }
    }
}

fn current_instrument_name(channel: u8, program: u8) -> String {
    // 채널이 타악기면 드럼 킷 이름을, 아니면 일반 GM 악기 이름을 반환한다.
    if channel == 9 {
        DRUM_KITS
            .get(&program)
            .copied()
            .unwrap_or("Standard Kit")
            .to_string()
    } else {
        MIDI_INSTRUMENTS
            .get(program as usize)
            .map(|instrument| instrument.name.to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}
