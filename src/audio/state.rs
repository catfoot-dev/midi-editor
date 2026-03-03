use std::collections::HashMap;

use crate::audio::midi_struct::MidiNote;

// 실제 오디오 처리 로직을 담은 '내부 상태' 구조체
pub struct SharedAudioState {
    pub is_playing: bool,
    pub is_repeat: bool,
    pub playback_cursor: usize,
    pub instruments: HashMap<u8, i32>,
    pub notes: Vec<MidiNote>,
}

impl SharedAudioState {
    pub fn new() -> Self {
        Self {
            is_playing: false,
            is_repeat: false,
            playback_cursor: 0,
            instruments: HashMap::new(),
            notes: vec![],
        }
    }
}
