use std::{env::consts::OS};

use egui::{Color32, Margin};

use crate::ui::{MidiApp, frame::Frame};

// 메뉴 예시
// +-------------------------------------------------------------+
// | File Edit View Help                                         |
// +-------------------------------------------------------------+

pub struct Menu {
}

impl Default for Menu {
    fn default() -> Self {
        Self {
        }
    }
}

impl Frame for Menu {
    const FRAME_NAME: &str = "Menu";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 23.0;
    const RESIZABLE: bool = false;

    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        let ctrl = if OS == "macos" { "⌘" } else { "Ctrl +" };

        egui::Frame::new()
            .inner_margin(Margin::same(2))
            .fill(Color32::from_rgb(32, 32, 32))
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        let open_btn = egui::Button::new("Open").shortcut_text(format!("{} O", ctrl));
                        if ui.add(open_btn).clicked() {
                            // 파일 열기 대화상자 표시
                            app.open_file();
                        }

                        let close_btn = egui::Button::new("Close").shortcut_text(format!("{} W", ctrl));
                        if ui.add(close_btn).clicked() {
                            // 파일 닫기
                            app.close();
                        }
                    });

                    ui.menu_button("Window", |ui| {
                        let track_list_show_btn = egui::Button::new("TrackList").shortcut_text(format!("{} T", ctrl));
                        if ui.add(track_list_show_btn).clicked() {
                            // 트랙리스트 창 표시
                            app.show_track_list = !app.show_track_list;
                        }

                        let attributes_show_btn = egui::Button::new("Attributes").shortcut_text(format!("{} A", ctrl));
                        if ui.add(attributes_show_btn).clicked() {
                            // 속성 창 표시
                            app.show_attributes = !app.show_attributes;
                        }

                        let keyboard_show_btn = egui::Button::new("Keyboard").shortcut_text(format!("{} K", ctrl));
                        if ui.add(keyboard_show_btn).clicked() {
                            // 키보드 창 표시
                            app.show_keyboard = !app.show_keyboard;
                        }
                    });
                });
            });
    }
}