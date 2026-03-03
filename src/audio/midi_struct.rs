
#[derive(Clone, Debug)]
pub struct MidiMeta {
    pub track_number: u16,
    pub text: String,
    pub copyright: String,
    pub program_name: String,
    pub tempo: f64,
    pub time_signature: [u8;4],
    pub key_signature: i8,
    pub is_minor: bool,
    pub track_name: String,
    pub marker: String,
}

#[derive(Clone, Debug)]
pub struct MidiTrack {
    pub name: String,
    pub channel: u8,
    pub instrument: u8,
    pub instrument_name: String,
    pub is_muted: bool,
    pub volume: f32,
}

#[derive(Clone, Debug)]
pub struct MidiData {
    pub is_on: bool,
    pub tick: u64,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub struct MidiNote {
    pub start_sample: usize,
    pub channel: i32,
    pub key: i32,
    pub velocity: i32,
}
