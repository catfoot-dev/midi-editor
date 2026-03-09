use std::{
    collections::{HashMap, VecDeque},
    fmt,
    path::Path,
};

use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};

use crate::audio::midi_struct::{
    NoteSpan, ProgramChangeEvent, Song, SongMeta, TempoChange, TrackModel,
    seconds_for_tick_with_tempo,
};

#[derive(Debug)]
pub enum MidiLoadError {
    Io(std::io::Error),
    Parse(midly::Error),
    UnsupportedTimecode,
}

impl fmt::Display for MidiLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "failed to read MIDI file: {error}"),
            Self::Parse(error) => write!(f, "failed to parse MIDI file: {error}"),
            Self::UnsupportedTimecode => write!(f, "timecode-based MIDI files are not supported"),
        }
    }
}

impl std::error::Error for MidiLoadError {}

pub struct LoadedSong {
    pub file_name: String,
    pub song: Song,
}

pub enum LoadResult {
    Success(LoadedSong),
    Error(String),
}

#[derive(Default)]
pub struct MidiManager {
    is_loading: bool,
    song: Option<Song>,
    pending_result: Option<LoadResult>,
}

impl MidiManager {
    /// 곡 로드가 끝났고 현재 참조 가능한 Song이 있는지 확인한다.
    pub fn is_loaded(&self) -> bool {
        !self.is_loading && self.song.is_some()
    }

    /// 백그라운드 로딩 중인지 UI가 확인할 때 사용한다.
    pub fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// 읽기 전용 Song 참조를 반환한다.
    pub fn song(&self) -> Option<&Song> {
        self.song.as_ref()
    }

    /// 트랙 설정 수정처럼 Song 내부를 갱신해야 할 때 사용한다.
    pub fn song_mut(&mut self) -> Option<&mut Song> {
        self.song.as_mut()
    }

    /// 새 파일 로드를 시작하기 전에 상태를 로딩 중으로 전환한다.
    pub fn begin_loading(&mut self) {
        self.is_loading = true;
        self.pending_result = None;
    }

    /// 백그라운드 스레드가 성공 결과를 전달할 때 호출된다.
    pub fn finish_loading(&mut self, file_name: String, song: Song) {
        self.is_loading = false;
        self.pending_result = Some(LoadResult::Success(LoadedSong { file_name, song }));
    }

    /// 백그라운드 스레드가 실패 원인을 전달할 때 호출된다.
    pub fn finish_loading_error(&mut self, error: String) {
        self.is_loading = false;
        self.pending_result = Some(LoadResult::Error(error));
    }

    /// 메인 스레드가 아직 처리하지 않은 로드 결과를 한 번만 꺼낸다.
    pub fn take_pending_result(&mut self) -> Option<LoadResult> {
        self.pending_result.take()
    }

    /// 성공적으로 로드된 Song을 현재 상태로 교체한다.
    pub fn apply_song(&mut self, song: Song) {
        self.song = Some(song);
    }

    /// 곡 닫기 시 로딩 상태와 현재 Song을 함께 비운다.
    pub fn close(&mut self) {
        self.is_loading = false;
        self.pending_result = None;
        self.song = None;
    }

    /// 파일 시스템에서 MIDI 파일을 읽고 Song으로 파싱한다.
    pub fn load_file(file_path: &Path) -> Result<Song, MidiLoadError> {
        let data = std::fs::read(file_path).map_err(MidiLoadError::Io)?;
        Self::parse_bytes(&data)
    }

    /// raw MIDI 바이트를 파싱해 트랙/tempo/note span 구조로 정규화한다.
    fn parse_bytes(data: &[u8]) -> Result<Song, MidiLoadError> {
        let smf = Smf::parse(data).map_err(MidiLoadError::Parse)?;
        let ppq = match smf.header.timing {
            Timing::Metrical(ticks) => ticks.as_int(),
            Timing::Timecode(_, _) => return Err(MidiLoadError::UnsupportedTimecode),
        };

        let mut meta = SongMeta::default();
        let mut tempo_changes = vec![TempoChange {
            tick: 0,
            micros_per_quarter: 500_000,
        }];
        let mut tracks = Vec::with_capacity(smf.tracks.len());
        let mut total_ticks = 0_u64;

        for (track_index, track) in smf.tracks.iter().enumerate() {
            let mut current_tick = 0_u64;
            let mut parsed_track = TrackAccumulator::new(track_index);

            for event in track {
                current_tick += event.delta.as_int() as u64;

                match event.kind {
                    TrackEventKind::Meta(message) => {
                        apply_meta_event(
                            message,
                            current_tick,
                            &mut meta,
                            &mut parsed_track,
                            &mut tempo_changes,
                        );
                    }
                    TrackEventKind::Midi { channel, message } => {
                        apply_midi_event(
                            channel.as_int(),
                            message,
                            current_tick,
                            &mut parsed_track,
                        );
                    }
                    _ => {}
                }
            }

            parsed_track.close_dangling_notes(current_tick);
            total_ticks = total_ticks.max(current_tick);
            tracks.push(parsed_track.finish());
        }

        // tempo 이벤트는 트랙 순서와 무관하게 절대 tick 기준으로 다시 정렬한다.
        let tempo_changes = normalize_tempo_changes(tempo_changes);
        let total_seconds = seconds_for_tick_with_tempo(ppq, &tempo_changes, total_ticks);

        Ok(Song {
            ppq,
            total_ticks,
            total_seconds,
            meta,
            tempo_changes,
            tracks,
        })
    }
}

struct TrackAccumulator {
    track_index: usize,
    name: String,
    channel: Option<u8>,
    program: u8,
    instrument_name: String,
    program_changes: Vec<ProgramChangeEvent>,
    note_spans: Vec<NoteSpan>,
    active_notes: HashMap<u8, VecDeque<ActiveNote>>,
}

impl TrackAccumulator {
    /// 파싱 중 사용할 임시 트랙 버퍼를 초기화한다.
    fn new(track_index: usize) -> Self {
        Self {
            track_index,
            name: format!("Track {}", track_index + 1),
            channel: None,
            program: 0,
            instrument_name: String::new(),
            program_changes: Vec::new(),
            note_spans: Vec::new(),
            active_notes: HashMap::new(),
        }
    }

    /// note off 또는 velocity 0 note on을 만나면 가장 앞선 note on과 짝을 맞춘다.
    fn close_note(&mut self, key: u8, end_tick: u64) {
        if let Some(notes) = self.active_notes.get_mut(&key)
            && let Some(active_note) = notes.pop_front()
        {
            self.note_spans.push(NoteSpan {
                key,
                start_tick: active_note.start_tick,
                end_tick,
                velocity: active_note.velocity,
            });
        }
    }

    /// 파일이 비정상적이더라도 열린 노트는 트랙 끝에서 닫아 시각화와 재생을 유지한다.
    fn close_dangling_notes(&mut self, end_tick: u64) {
        for (key, active_notes) in &mut self.active_notes {
            while let Some(active_note) = active_notes.pop_front() {
                self.note_spans.push(NoteSpan {
                    key: *key,
                    start_tick: active_note.start_tick,
                    end_tick,
                    velocity: active_note.velocity,
                });
            }
        }
    }

    /// 정렬과 기본 UI 상태를 마무리한 뒤 최종 TrackModel로 변환한다.
    fn finish(mut self) -> TrackModel {
        self.note_spans
            .sort_by_key(|note| (note.start_tick, note.end_tick, note.key));
        self.program_changes
            .sort_by_key(|program_change| program_change.tick);

        TrackModel {
            track_index: self.track_index,
            name: self.name,
            channel: self.channel.unwrap_or(0),
            program: self.program,
            instrument_name: self.instrument_name,
            program_changes: self.program_changes,
            note_spans: self.note_spans,
            is_muted: false,
            volume: 100.0,
        }
    }
}

struct ActiveNote {
    start_tick: u64,
    velocity: u8,
}

// 메타 이벤트를 곡 전체 메타데이터와 트랙 메타데이터로 분배한다.
fn apply_meta_event(
    message: MetaMessage<'_>,
    current_tick: u64,
    meta: &mut SongMeta,
    track: &mut TrackAccumulator,
    tempo_changes: &mut Vec<TempoChange>,
) {
    match message {
        MetaMessage::TrackName(data) => {
            let value = sanitize_text(data);
            if !value.is_empty() {
                track.name = value.clone();
                if meta.title.is_empty() {
                    meta.title = value;
                }
            }
        }
        MetaMessage::InstrumentName(data) => {
            let value = sanitize_text(data);
            if !value.is_empty() {
                track.instrument_name = value;
            }
        }
        MetaMessage::ProgramName(data) => {
            let value = sanitize_text(data);
            if meta.program_name.is_empty() {
                meta.program_name = value;
            }
        }
        MetaMessage::Text(data) => {
            let value = sanitize_text(data);
            if meta.text.is_empty() {
                meta.text = value;
            }
        }
        MetaMessage::Copyright(data) => {
            let value = sanitize_text(data);
            if meta.copyright.is_empty() {
                meta.copyright = value;
            }
        }
        MetaMessage::Marker(data) => {
            let value = sanitize_text(data);
            if meta.marker.is_empty() {
                meta.marker = value;
            }
        }
        MetaMessage::Tempo(tempo) => {
            tempo_changes.push(TempoChange {
                tick: current_tick,
                micros_per_quarter: tempo.as_int(),
            });
        }
        MetaMessage::TimeSignature(nn, dd, cc, bb) => {
            meta.time_signature = [nn, dd, cc, bb];
        }
        MetaMessage::KeySignature(sf, mi) => {
            meta.key_signature = sf;
            meta.is_minor = mi;
        }
        _ => {}
    }
}

// MIDI 이벤트를 프로그램 체인지와 note span 정보로 축약한다.
fn apply_midi_event(
    channel: u8,
    message: MidiMessage,
    current_tick: u64,
    track: &mut TrackAccumulator,
) {
    track.channel.get_or_insert(channel);

    match message {
        MidiMessage::ProgramChange { program } => {
            let program = program.as_int();
            if track.program_changes.is_empty() {
                // UI 기본값은 첫 프로그램 체인지 기준으로 잡고,
                // 이후 변경은 program_changes에 시간축 정보로 남긴다.
                track.program = program;
            }
            track.program_changes.push(ProgramChangeEvent {
                tick: current_tick,
                program,
            });
        }
        MidiMessage::NoteOn { key, vel } if vel.as_int() > 0 => {
            // 같은 키가 겹칠 수 있으므로 큐에 쌓아 두고 note off에서 순서대로 닫는다.
            track
                .active_notes
                .entry(key.as_int())
                .or_default()
                .push_back(ActiveNote {
                    start_tick: current_tick,
                    velocity: vel.as_int(),
                });
        }
        MidiMessage::NoteOn { key, .. } | MidiMessage::NoteOff { key, .. } => {
            track.close_note(key.as_int(), current_tick);
        }
        _ => {}
    }
}

// 텍스트 메타 이벤트에서 널 문자와 앞뒤 공백을 제거한다.
fn sanitize_text(data: &[u8]) -> String {
    String::from_utf8_lossy(data)
        .replace('\0', "")
        .trim()
        .to_string()
}

// tempo map을 tick 기준 오름차순으로 정렬하고 같은 tick의 중복은 마지막 값으로 압축한다.
fn normalize_tempo_changes(mut tempo_changes: Vec<TempoChange>) -> Vec<TempoChange> {
    tempo_changes.sort_by_key(|tempo_change| tempo_change.tick);

    let mut normalized: Vec<TempoChange> = Vec::with_capacity(tempo_changes.len());
    for tempo_change in tempo_changes {
        if let Some(previous) = normalized.last_mut()
            && previous.tick == tempo_change.tick
        {
            previous.micros_per_quarter = tempo_change.micros_per_quarter;
            continue;
        }
        normalized.push(tempo_change);
    }

    if normalized.is_empty() || normalized[0].tick != 0 {
        normalized.insert(
            0,
            TempoChange {
                tick: 0,
                micros_per_quarter: 500_000,
            },
        );
    }

    normalized
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use midly::{
        Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
        num::{u4, u7, u15, u24, u28},
    };

    use super::MidiManager;

    #[test]
    fn parses_zero_velocity_note_on_as_note_off() {
        let song = MidiManager::parse_bytes(&write_smf(Smf {
            header: Header::new(Format::SingleTrack, Timing::Metrical(u15::from(480))),
            tracks: vec![vec![
                TrackEvent {
                    delta: u28::from(0),
                    kind: TrackEventKind::Midi {
                        channel: u4::from(0),
                        message: MidiMessage::NoteOn {
                            key: u7::from(60),
                            vel: u7::from(100),
                        },
                    },
                },
                TrackEvent {
                    delta: u28::from(480),
                    kind: TrackEventKind::Midi {
                        channel: u4::from(0),
                        message: MidiMessage::NoteOn {
                            key: u7::from(60),
                            vel: u7::from(0),
                        },
                    },
                },
            ]],
        }))
        .unwrap();

        assert_eq!(song.tracks.len(), 1);
        assert_eq!(song.tracks[0].note_spans.len(), 1);
        assert_eq!(song.tracks[0].note_spans[0].start_tick, 0);
        assert_eq!(song.tracks[0].note_spans[0].end_tick, 480);
    }

    #[test]
    fn keeps_multiple_tracks_on_same_channel_distinct() {
        let song = MidiManager::parse_bytes(&write_smf(Smf {
            header: Header::new(Format::Parallel, Timing::Metrical(u15::from(480))),
            tracks: vec![
                vec![
                    TrackEvent {
                        delta: u28::from(0),
                        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"Track A")),
                    },
                    TrackEvent {
                        delta: u28::from(0),
                        kind: TrackEventKind::Midi {
                            channel: u4::from(0),
                            message: MidiMessage::NoteOn {
                                key: u7::from(60),
                                vel: u7::from(100),
                            },
                        },
                    },
                    TrackEvent {
                        delta: u28::from(240),
                        kind: TrackEventKind::Midi {
                            channel: u4::from(0),
                            message: MidiMessage::NoteOff {
                                key: u7::from(60),
                                vel: u7::from(0),
                            },
                        },
                    },
                ],
                vec![
                    TrackEvent {
                        delta: u28::from(0),
                        kind: TrackEventKind::Meta(MetaMessage::TrackName(b"Track B")),
                    },
                    TrackEvent {
                        delta: u28::from(0),
                        kind: TrackEventKind::Midi {
                            channel: u4::from(0),
                            message: MidiMessage::NoteOn {
                                key: u7::from(64),
                                vel: u7::from(96),
                            },
                        },
                    },
                    TrackEvent {
                        delta: u28::from(240),
                        kind: TrackEventKind::Midi {
                            channel: u4::from(0),
                            message: MidiMessage::NoteOff {
                                key: u7::from(64),
                                vel: u7::from(0),
                            },
                        },
                    },
                ],
            ],
        }))
        .unwrap();

        assert_eq!(song.tracks.len(), 2);
        assert_eq!(song.tracks[0].name, "Track A");
        assert_eq!(song.tracks[1].name, "Track B");
        assert_eq!(song.tracks[0].channel, 0);
        assert_eq!(song.tracks[1].channel, 0);
    }

    #[test]
    fn sorts_tempo_changes_before_converting_song_length() {
        let song = MidiManager::parse_bytes(&write_smf(Smf {
            header: Header::new(Format::Parallel, Timing::Metrical(u15::from(480))),
            tracks: vec![
                vec![TrackEvent {
                    delta: u28::from(480),
                    kind: TrackEventKind::Meta(MetaMessage::Tempo(u24::from(1_000_000))),
                }],
                vec![TrackEvent {
                    delta: u28::from(240),
                    kind: TrackEventKind::Midi {
                        channel: u4::from(0),
                        message: MidiMessage::NoteOn {
                            key: u7::from(60),
                            vel: u7::from(100),
                        },
                    },
                }],
            ],
        }))
        .unwrap();

        assert_eq!(song.tempo_changes[0].tick, 0);
        assert_eq!(song.tempo_changes[1].tick, 480);
        assert!((song.total_seconds - 0.5).abs() < f64::EPSILON);
    }

    fn write_smf(smf: Smf<'static>) -> Vec<u8> {
        let mut out = Cursor::new(Vec::new());
        smf.write_std(&mut out).unwrap();
        out.into_inner()
    }
}
