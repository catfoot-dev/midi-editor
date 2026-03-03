use std::collections::HashMap;
use midly::{MetaMessage, MidiMessage, Timing, TrackEventKind};

use crate::audio::midi_struct::{MidiData, MidiMeta, MidiTrack};

pub struct MidiManager {
    is_parsing: bool,
    data: Vec<u8>,
    pub total_seconds: f64,
    pub ppq: f64,
    pub solo_track: Option<u8>,
    pub meta: MidiMeta,
    pub midi: HashMap<u8, HashMap<u8, Vec<MidiData>>>,
    pub tracks: HashMap<u8, MidiTrack>,
}

impl Default for MidiManager {
    fn default() -> Self {
        Self {
            is_parsing: false,
            data: Vec::new(),
            total_seconds: 0.0,
            ppq: 0.0,
            solo_track: None,
            meta: MidiMeta {
                track_number: 0,
                text: String::new(),
                copyright: String::new(),
                program_name: String::new(),
                tempo: 0.0,
                time_signature: [0,0,0,0],
                key_signature: 0,
                is_minor: true,
                track_name: String::new(),
                marker: String::new(),
            },
            midi: HashMap::new(),
            tracks: HashMap::new(),
        }
    }
}

impl MidiManager {
    pub fn is_loaded(&mut self) -> bool {
        return !self.is_parsing && self.data.len() > 0;
    }

    pub fn open(&mut self, file_path: &str) {
        if self.is_parsing {
            return;
        }

        self.data = std::fs::read(file_path).unwrap();
        let smf = midly::Smf::parse(&self.data).unwrap();

        self.ppq = match smf.header.timing {
            Timing::Metrical(ticks) => ticks.as_int() as f64,
            Timing::Timecode(_, _) => 0.0,
        };

        self.meta.tempo = 500_000f64;
        let mut tempo_changes = vec![(0u64, self.meta.tempo)];
        let mut max_absolute_tick = 0u64;
        
        for track in smf.tracks.iter() {
            let mut current_tick = 0u64;

            for event in track.iter() {
                current_tick += event.delta.as_int() as u64;
                
                match event.kind {
                    TrackEventKind::Meta(meta) => {
                        match meta {
                            MetaMessage::TrackName(data) => {
                                // 트랙 이름
                                self.meta.track_name = String::from_utf8_lossy(data).into_owned().replace("\0", "");
                                // println!("TrackName: {:?}", self.meta.track_name);
                            }
                            MetaMessage::InstrumentName(data) => {
                                let instrument_name = String::from_utf8_lossy(data).into_owned().replace("\0", "");
                                println!("Intrument Name: {:?}", instrument_name);
                                // self.tracks.get()
                            }
                            MetaMessage::ProgramName(program_name) => {
                                // 프로그램 이름
                                self.meta.program_name = String::from_utf8_lossy(program_name).into_owned().replace("\0", "");
                                // println!("Program Name: {:#?}", self.meta.program_name);
                            }
                            MetaMessage::Tempo(tempo) => {
                                // 템포
                                self.meta.tempo = tempo.as_int() as f64;
                                tempo_changes.push((current_tick, tempo.as_int() as f64));
                            }
                            MetaMessage::TimeSignature(nn, dd, cc, bb) => {
                                // 박자
                                self.meta.time_signature = [nn, dd, cc, bb];
                                // println!("Time Signature: {}/{} (Clocks per click: {}, 32nd notes per quarter note: {})",
                                //     nn, 2u8.pow(dd as u32), cc, bb);
                            }
                            MetaMessage::KeySignature(sf, mi) => {
                                // 조표: minor 단조, major 장조
                                self.meta.is_minor = mi;
                                self.meta.key_signature = sf;
                                // let mode = if mi { "Minor" } else { "Major" };
                                // println!("Key Signature: {} {}, Sharps/Flats: {}", sf, mode,
                                //     if sf > 0 { format!("{} Sharps", sf) }
                                //     else if sf < 0 { format!("{} Flats", -sf) }
                                //     else { "No Sharps or Flats".to_string() });
                            }
                            MetaMessage::Unknown(cue_point, data) => {
                                if cue_point == 0x0A {
                                    let writer = String::from_utf8_lossy(data).into_owned().replace("\0", "");
                                    println!("Writer: {}", writer);
                                } else {
                                    println!("Unknown Meta Message: Cue Point: {}, Data: {:#?}", cue_point, data);
                                }
                            }
                            _ => println!("event - meta: {:?}", event)
                        }
                    }
                    TrackEventKind::Midi { channel, message } => {
                        let channel = channel.as_int();
                        if !self.midi.contains_key(&channel) {
                            self.midi.insert(channel, HashMap::new());
                        }
                        let midi = self.midi.get_mut(&channel).unwrap();

                        if !self.tracks.contains_key(&channel) {
                            let track = MidiTrack {
                                name: "Track".to_string(),
                                channel,
                                instrument: 0,
                                instrument_name: String::new(),
                                is_muted: false,
                                volume: 100.0,
                            };
                            self.tracks.insert(channel, track);
                        }

                        match message {
                            MidiMessage::ProgramChange { program } => {
                                // 채널 별 악기
                                self.tracks.get_mut(&channel).unwrap().instrument = program.as_int();
                            }
                            MidiMessage::NoteOn { key, vel } => {
                                // 노트 온 (velocity 값이 0으로 오프가 되는 경우도 있음)
                                let key = key.as_int();
                                let data = MidiData {
                                    is_on: true,
                                    tick: current_tick,
                                    velocity: vel.as_int(),
                                };
                                if !midi.contains_key(&key) {
                                    midi.insert(key, vec![data]);
                                } else {
                                    midi.get_mut(&key).unwrap().push(data);
                                }
                            }
                            MidiMessage::NoteOff { key, vel } => {
                                // 노트 오프
                                let key = key.as_int();
                                let data = MidiData {
                                    is_on: false,
                                    tick: current_tick,
                                    velocity: vel.as_int(),
                                };
                                if !midi.contains_key(&key) {
                                    midi.insert(key, vec![data]);
                                } else {
                                    midi.get_mut(&key).unwrap().push(data);
                                }
                            }
                            _ => println!("event - midi: {:?}", event)
                        }
                    }
                    _ => println!("event: {:?}", event)
                }
            }

            // 전체 플레이 시간 계산
            if current_tick > max_absolute_tick {
                max_absolute_tick = current_tick;
            }

            self.total_seconds = 0.0;
            let mut last_tick = 0u64;
            let mut current_tempo = tempo_changes[0].1;

            for &(tick, tempo) in &tempo_changes {
                if tick > max_absolute_tick { break; }

                let ticks_passed = tick - last_tick;

                let seconds_per_tick = current_tempo / 1_000_000.0 / self.ppq;
                self.total_seconds += ticks_passed as f64 * seconds_per_tick;

                last_tick = tick;
                current_tempo = tempo;
            }

            if max_absolute_tick > last_tick {
                let ticks_passed = max_absolute_tick - last_tick;
                let seconds_per_tick = current_tempo / 1_000_000.0 / self.ppq;
                self.total_seconds += ticks_passed as f64 * seconds_per_tick;
            }
        }
    }

    pub fn close(&mut self) {
        self.is_parsing = false;
        self.data = Vec::new();
        self.total_seconds = 0.0;
        self.ppq = 0.0;
        self.solo_track = None;
        self.meta = MidiMeta {
            track_number: 0,
            program_name: String::new(),
            tempo: 0.0,
            time_signature: [0,0,0,0],
            key_signature: 0,
            is_minor: true,
            text: String::new(),
            copyright: String::new(),
            track_name: String::new(),
            marker: String::new(),
        };
        self.tracks = HashMap::new();
        self.midi = HashMap::new();
    }
}
