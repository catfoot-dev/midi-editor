use crate::ui::{MidiApp, frame::Frame};

// 선택된 노트/이벤트 속성 예시
// +-----------------------------+
// | Attributes                  |
// +-----------------------------+
// | Pitch: 60 (C4)              |
// | Velocity: 100               |
// | Duration: 480 ticks         |
// | Start Time: 960 ticks       |
// |             ...             |
// +-----------------------------+

pub struct Attributes {
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
        }
    }
}

impl Frame for Attributes {
    const FRAME_NAME: &str = "Attributes";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 225.0;
    const HEIGHT: f32 = 0.0;
    const RESIZABLE: bool = false;

    fn draw(&mut self, ui: &mut egui::Ui, _app: &mut MidiApp) {
        self.header(ui);

        ui.scope_builder(egui::UiBuilder::new(), |ui| {
            egui::Grid::new(Attributes::FRAME_NAME)
                .num_columns(2)
                .spacing([20.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.add_sized([80.0, 22.0], egui::Label::new("Name"));
                    ui.add_sized([110.0, 22.0], egui::Label::new("Value"));
                    ui.end_row();

                    let mut pitch = format!("{}", 60);
                    ui.add(egui::Label::new("Pitch"));
                    let edit_box = egui::TextEdit::singleline(&mut pitch)
                        .desired_width(40.0);
                    if ui.add(edit_box).changed() {
                        // pitch;
                    }
                    ui.end_row();

                    ui.add(egui::Label::new("Velocity"));
                    let mut velocity = format!("{}", 100);
                    let edit_box = egui::TextEdit::singleline(&mut velocity)
                        .desired_width(40.0);
                    if ui.add(edit_box).changed() {
                        // velocity;
                    }
                    ui.end_row();

                    ui.add(egui::Label::new("Duration"));
                    let mut duration = format!("{}", 480);
                    ui.horizontal(|ui| {
                        let edit_box = egui::TextEdit::singleline(&mut duration)
                            .desired_width(40.0);
                        if ui.add(edit_box).changed() {
                            // duration;
                        }
                        ui.label("ticks");
                    });
                    ui.end_row();

                    ui.add(egui::Label::new("Start Time"));
                    let mut start_time = format!("{}", 960);
                    ui.horizontal(|ui| {
                        let edit_box = egui::TextEdit::singleline(&mut start_time)
                            .desired_width(40.0);
                        if ui.add(edit_box).changed() {
                            // start_time;
                        }
                        ui.label("ticks");
                    });
                    ui.end_row();
                });
        });
    }
}
