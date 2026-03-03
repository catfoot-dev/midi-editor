pub mod audio;
pub mod midi;
pub mod ui;

fn main() {
    ui::MidiApp::run().unwrap();
}
