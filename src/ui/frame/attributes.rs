use crate::ui::{frame::Frame, MidiApp};

#[derive(Default)]
pub struct Attributes;

impl Frame for Attributes {
    const FRAME_NAME: &str = "Attributes";
    const INNER_MARGIN: egui::Margin = egui::Margin::same(0);
    const WIDTH: f32 = 225.0;
    const HEIGHT: f32 = 0.0;
    const RESIZABLE: bool = false;

    // 현재 선택된 트랙의 요약 정보를 읽기 전용으로 보여 준다.
    fn draw(&mut self, ui: &mut egui::Ui, app: &mut MidiApp) {
        self.header(ui);

        let midi_manager = app.midi_manager.lock().unwrap();
        let Some(song) = midi_manager.song() else {
            ui.label("No track selected.");
            return;
        };

        let Some(selected_track_index) = app.select_track else {
            ui.label("Select a track to inspect.");
            return;
        };

        let Some(track) = song
            .tracks
            .iter()
            .find(|track| track.track_index == selected_track_index)
        else {
            ui.label("Selected track is no longer available.");
            return;
        };

        egui::Grid::new(Attributes::FRAME_NAME)
            .num_columns(2)
            .spacing([20.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Name");
                ui.label(track.display_name());
                ui.end_row();

                ui.label("Channel");
                ui.label(format!("{:02}", track.channel + 1));
                ui.end_row();

                ui.label("Program");
                ui.label(format!("{}", track.program));
                ui.end_row();

                ui.label("Notes");
                ui.label(format!("{}", track.note_spans.len()));
                ui.end_row();

                ui.label("Range");
                let range = track
                    .note_spans
                    .iter()
                    .map(|note| note.key)
                    .min()
                    .zip(track.note_spans.iter().map(|note| note.key).max())
                    .map(|(min, max)| format!("{min}..{max}"))
                    .unwrap_or_else(|| "-".to_string());
                ui.label(range);
                ui.end_row();
            });
    }
}
