pub mod audio;
pub mod midi;
pub mod ui;

fn main() {
    // 초기화 실패는 즉시 종료하되 패닉 대신 오류 메시지를 표준 에러로 남긴다.
    if let Err(error) = ui::MidiApp::run() {
        eprintln!("Failed to start MIDI Editor: {error}");
    }
}
