use std::env::consts::OS;

use egui::{Color32, Margin};

use crate::ui::{frame::Frame, MidiApp};

#[derive(Default)]
pub struct Menu;

impl Frame for Menu {
    const FRAME_NAME: &str = "Menu";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 0.0;
    const HEIGHT: f32 = 23.0;
    const RESIZABLE: bool = false;

    // 상단 메뉴에서 파일 열기/닫기와 패널 표시 토글을 처리한다.
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        let ctrl = if OS == "macos" { "⌘" } else { "Ctrl +" };
        let is_loading = app.midi_manager.lock().unwrap().is_loading();

        egui::Frame::new()
            .inner_margin(Margin::same(2))
            .fill(Color32::from_rgb(32, 32, 32))
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        let open_btn =
                            egui::Button::new("Open").shortcut_text(format!("{} O", ctrl));
                        if ui.add_enabled(!is_loading, open_btn).clicked() {
                            app.open_file();
                        }

                        let close_btn =
                            egui::Button::new("Close").shortcut_text(format!("{} W", ctrl));
                        if ui.add_enabled(!is_loading, close_btn).clicked() {
                            app.close();
                        }
                    });

                    ui.menu_button("Window", |ui| {
                        let track_list_btn = egui::Button::new("TrackList")
                            .shortcut_text(format!("{} T", ctrl));
                        if ui.add(track_list_btn).clicked() {
                            app.show_track_list = !app.show_track_list;
                        }

                        let attributes_btn = egui::Button::new("Attributes")
                            .shortcut_text(format!("{} A", ctrl));
                        if ui.add(attributes_btn).clicked() {
                            app.show_attributes = !app.show_attributes;
                        }

                        let keyboard_btn = egui::Button::new("Keyboard")
                            .shortcut_text(format!("{} K", ctrl));
                        if ui.add(keyboard_btn).clicked() {
                            app.show_keyboard = !app.show_keyboard;
                        }
                    });
                });
            });
    }
}
