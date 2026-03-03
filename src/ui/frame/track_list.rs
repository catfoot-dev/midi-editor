use egui::Color32;

use crate::{midi::instruments::{DRUM_KITS, MIDI_GROUPS, MIDI_INSTRUMENTS}, ui::{MidiApp, frame::Frame}};

// 트랙 리스트
// 1. 축소 예시
// +------------------+----+------+------+--------------+
// | Track            | CH | Solo | Mute | Volume       |
// +------------------+----+------+------+--------------+
// | ▶ 1 Piano Melody | 01 | [ ]  | [x]  | ------O- 80% |
// | ▶ 2 Piano Melody | 01 | [x]  | [ ]  | -------O100% |
// | ▶ 3 Piano Melody | 01 | [ ]  | [ ]  | ------O- 80% |
// |                         ...                        |
// +----------------------------------------------------+
// 2. 확장 예시
// | ▶ 1 Piano Melody | 01 | [ ]  | [x]  | ------O- 80% |
// +------------------+----+------+------+--------------+
// |  Solo: [ ]     Mute: [x]    Volume: ------O- 80%   |
// |  CH: [ 01 ▼ ]  Instrument: [ Piano Melody ]        |
// +------------------+----+------+------+--------------+
// | ▶ 2 Piano Melody | 01 | [x]  | [ ]  | -------O100% |

pub struct TrackList {
}

impl Default for TrackList {
    fn default() -> Self {
        Self {
        }
    }
}

impl Frame for TrackList {
    const FRAME_NAME: &str = "TrackList";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 225.0;
    const HEIGHT: f32 = 0.0;
    const RESIZABLE: bool = false;

    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        use egui_extras::{Column, TableBuilder};

        self.header(ui);

        // 테이블 빌더 설정
        let available_height = ui.available_height();
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(210.0))
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height)
            .header(20.0, |mut header| {
                // 헤더 설정
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
    fn set_track(&mut self, body: &mut egui_extras::TableBody<'_>, app: &mut MidiApp) {
        let mut midi_manager = app.midi_manager.lock().unwrap();
        if !midi_manager.is_loaded() { return; }

        // println!("Track: {:?}", self.smf.as_ref().unwrap());
        for (id, track) in midi_manager.tracks.iter_mut() {
            // for event in track.iter_mut(){
            //     println!("{:?}", event);
            // }

            let is_extended = app.select_track == Some(*id);
            let row_height = if is_extended { 96.0 } else { 20.0 };
            let instrument = MIDI_INSTRUMENTS.get(track.instrument as usize)
                .unwrap();
            let drum = match DRUM_KITS.get(&track.instrument) {
                Some(value) => value,
                None => DRUM_KITS.get(&0u8).unwrap(),
            };

            body.row(row_height, |mut row| {
                row.col(|ui| {
                    let label_width = 52.0;
                    let row_height = ui.text_style_height(&egui::TextStyle::Body);

                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            // 트랙 정보 자세히 보기 버튼
                            let btn_text = if is_extended { "-" } else { "+" };
                            let accordion_btn = egui::Button::new(btn_text).small();
                            let accordion_btn_ui = ui.add_sized([20.0, row_height], accordion_btn)
                                .on_hover_text("Details");
                            if accordion_btn_ui.clicked() {
                                app.select_track = if app.select_track.is_none() || !is_extended {
                                    Some(*id)
                                } else {
                                    None
                                };
                            }

                            // 트랙 정보
                            let instrument_name = if track.channel != 10 { instrument.name } else { drum };
                            let track_label = egui::Label::new(format!("{} {}", track.instrument, instrument_name));
                            ui.add_sized([100.0, row_height], track_label);

                            // 채널 정보
                            let channel_label = egui::Label::new(format!("{:0>2}", track.channel));
                            ui.add_sized([20.0, row_height], channel_label);

                            // 솔로 체크박스
                            let mut is_solo = app.solo_track == Some(*id);
                            if ui.checkbox(&mut is_solo, "").clicked() {
                                app.solo_track = if is_solo { Some(*id) } else { None };
                            }

                            // 뮤트 체크박스
                            ui.checkbox(&mut track.is_muted, "");
                        });

                        if !is_extended { return; }
                        // 선택한 행의 경우 아래 확장 내용을 표시함

                        ui.separator();

                        // channel, track name
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            // 채널 드롭박스
                            ui.add_sized([label_width, row_height], egui::Label::new("Channel:"));
                            egui::ComboBox::from_id_salt("channel")
                                .width(40.0)
                                .selected_text(format!("{:0>2}", track.channel))
                                .show_ui(ui, |ui| {
                                    for ch in 0..=16 {
                                        ui.selectable_value(&mut track.channel, ch, format!("{:0>2}", ch));
                                    }
                                });
                        });

                        // instrument
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            // 악기 드롭박스
                            ui.add_sized([label_width, row_height], egui::Label::new("Instrument:"));
                            let mut selected_text = instrument.name;
                            let mut instruments: Vec<(u8, String)> = Vec::new();
                            if track.channel != 10 {
                                // 채널 10이 아닐 경우 일반 악기
                                for (idx, instrument) in MIDI_INSTRUMENTS.iter().enumerate() {
                                    let group = MIDI_GROUPS.get(instrument.group as usize).unwrap();
                                    let select_option_name = format!("{}. [{}] {}", idx, group.name, instrument.name);
                                    instruments.push((idx as u8, select_option_name));
                                }
                            } else {
                                // 채널 10일 경우 타악기
                                selected_text = drum;
                                for (idx, instrument) in DRUM_KITS.into_iter() {
                                    let select_option_name = format!("{}. {}", idx, instrument);
                                    instruments.push((*idx as u8, select_option_name));
                                }
                                instruments.sort();
                            }
                            egui::ComboBox::from_id_salt("instrument")
                                .width(140.0)
                                .selected_text(format!("{}", selected_text))
                                .show_ui(ui, |ui| {
                                    for (idx, instrument) in instruments {
                                        ui.selectable_value(&mut track.instrument, idx as u8, instrument);
                                    }
                                });
                        });

                        // volume
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                            // 볼륨 슬라이더
                            ui.add_sized([label_width, row_height], egui::Label::new("Volume:"));
                            ui.horizontal(|ui| {
                                let slider = egui::Slider::new(&mut track.volume, 0.0..=100.0)
                                    .show_value(false);
                                ui.add_enabled(!track.is_muted, slider);

                                // 볼륨 퍼센테이지
                                let font_id = egui::FontId::new(12.0, egui::FontFamily::default());
                                let volume_text = format!("{}%", track.volume as u8);
                                let volume_pos = ui.min_rect().center();
                                ui.painter().text(
                                    volume_pos, egui::Align2::CENTER_CENTER,
                                    volume_text, font_id, Color32::GRAY,
                                );
                            });
                        });

                        ui.separator();
                    });
                });
            });
        }
    }
}
