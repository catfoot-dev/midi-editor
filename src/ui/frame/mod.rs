use egui::{Color32, Margin};

use crate::ui::MidiApp;

pub mod attributes;
pub mod keyboard;
pub mod menu;
pub mod note_grid;
pub mod track_list;
pub mod transport;

const FRAME_HEAD_HEIGHT: f32 = 24.0; // 프레임 헤더 높이: 24px

pub trait Frame {
    const FRAME_NAME: &str;
    const INNER_MARGIN: egui::Margin;
    const WIDTH: f32;
    const HEIGHT: f32;
    const RESIZABLE: bool;
    
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp);

    fn header(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .inner_margin(Margin { top: 4, left: 6, right: 6, bottom: 4 })
            .fill(Color32::from_rgb(32, 32, 32))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.visuals_mut().override_text_color = Some(Color32::WHITE);
                ui.label(Self::FRAME_NAME.to_uppercase());
            });
    }
}
