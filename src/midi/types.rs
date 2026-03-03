pub struct Note {
    pub key: u8,
    pub start_tick: u32,
    pub duration: u32,
    pub velocity: u8,
    pub selected: bool,
}

pub struct Track {
    pub name: String,
    pub notes: Vec<Note>,
    pub channel: u8,
    pub mute: bool,
    pub solo: bool,
}
