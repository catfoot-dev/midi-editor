use std::sync::Arc;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SongMeta {
    pub title: String,
    pub text: String,
    pub copyright: String,
    pub program_name: String,
    pub time_signature: [u8; 4],
    pub key_signature: i8,
    pub is_minor: bool,
    pub marker: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TempoChange {
    pub tick: u64,
    pub micros_per_quarter: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramChangeEvent {
    pub tick: u64,
    pub program: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoteSpan {
    pub key: u8,
    pub start_tick: u64,
    pub end_tick: u64,
    pub velocity: u8,
}

#[derive(Clone, Debug)]
pub struct TrackModel {
    pub track_index: usize,
    pub name: String,
    pub channel: u8,
    pub program: u8,
    pub instrument_name: String,
    pub program_changes: Vec<ProgramChangeEvent>,
    pub note_spans: Vec<NoteSpan>,
    pub is_muted: bool,
    pub volume: f32,
}

impl TrackModel {
    /// 트랙 이름이 비어 있으면 악기 이름을 대체 표기로 사용한다.
    pub fn display_name(&self) -> &str {
        if self.name.is_empty() {
            &self.instrument_name
        } else {
            &self.name
        }
    }

    /// General MIDI 규약의 타악기 채널(10번, 0-based로는 9번) 여부를 반환한다.
    pub fn is_percussion(&self) -> bool {
        self.channel == 9
    }
}

#[derive(Clone, Debug, Default)]
pub struct Song {
    pub ppq: u16,
    pub total_ticks: u64,
    pub total_seconds: f64,
    pub meta: SongMeta,
    pub tempo_changes: Vec<TempoChange>,
    pub tracks: Vec<TrackModel>,
}

impl Song {
    /// 로드된 곡에 실제 트랙 데이터가 있는지 확인한다.
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    /// 상단 UI에서 표시할 대표 제목을 선택한다.
    pub fn display_title(&self) -> Option<&str> {
        if !self.meta.title.is_empty() {
            Some(self.meta.title.as_str())
        } else if !self.meta.program_name.is_empty() {
            Some(self.meta.program_name.as_str())
        } else {
            None
        }
    }

    /// tick 값을 그리드 렌더링에 사용할 beat 단위로 변환한다.
    pub fn beats_for_ticks(&self, ticks: u64) -> f32 {
        if self.ppq == 0 {
            0.0
        } else {
            ticks as f32 / self.ppq as f32
        }
    }

    /// tempo map을 기준으로 특정 tick의 실제 시간을 계산한다.
    pub fn seconds_for_tick(&self, tick: u64) -> f64 {
        seconds_for_tick_with_tempo(self.ppq, &self.tempo_changes, tick)
    }

    /// 재생 스케줄링을 위해 tick을 샘플 인덱스로 변환한다.
    pub fn sample_index_for_tick(&self, tick: u64, sample_rate: u32) -> usize {
        (self.seconds_for_tick(tick) * sample_rate as f64).round() as usize
    }

    /// 현재 재생 샘플 위치를 다시 MIDI tick으로 역변환한다.
    pub fn tick_for_sample(&self, sample_index: usize, sample_rate: u32) -> u64 {
        if sample_rate == 0 {
            return 0;
        }

        tick_for_seconds_with_tempo(
            self.ppq,
            &self.tempo_changes,
            sample_index as f64 / sample_rate as f64,
        )
    }

    /// 현재 트랙 상태를 바탕으로 오디오 스레드가 읽을 재생 이벤트 목록을 만든다.
    pub fn playback_data(&self, sample_rate: u32, solo_track: Option<usize>) -> PlaybackData {
        let mut events = Vec::new();

        for track in self.tracks.iter().filter(|track| match solo_track {
            Some(track_index) => track.track_index == track_index && !track.is_muted,
            None => !track.is_muted,
        }) {
            let mut program_changes = track.program_changes.clone();
            if program_changes.is_empty() || program_changes[0].tick != 0 {
                // 파일에 프로그램 체인지가 없더라도 시작 시점 악기는 항상 확정해 둔다.
                program_changes.insert(
                    0,
                    ProgramChangeEvent {
                        tick: 0,
                        program: track.program,
                    },
                );
            }

            for program_change in &program_changes {
                events.push(PlaybackEvent {
                    sample_index: self.sample_index_for_tick(program_change.tick, sample_rate),
                    channel: track.channel,
                    kind: PlaybackEventKind::ProgramChange {
                        program: program_change.program,
                    },
                });
            }

            for note in &track.note_spans {
                let start_sample = self.sample_index_for_tick(note.start_tick, sample_rate);
                let end_sample = self.sample_index_for_tick(note.end_tick, sample_rate);
                events.push(PlaybackEvent {
                    sample_index: start_sample,
                    channel: track.channel,
                    kind: PlaybackEventKind::NoteOn {
                        key: note.key,
                        velocity: note.velocity,
                    },
                });
                events.push(PlaybackEvent {
                    sample_index: end_sample,
                    channel: track.channel,
                    kind: PlaybackEventKind::NoteOff { key: note.key },
                });
            }
        }

        // 같은 시점에서는 ProgramChange -> NoteOff -> NoteOn 순서로 처리해야 음 끊김이 덜하다.
        events.sort_by_key(|event| {
            (
                event.sample_index,
                event.kind.sort_priority(),
                event.channel,
                event.kind.note_key(),
            )
        });

        PlaybackData {
            song_length_samples: self.sample_index_for_tick(self.total_ticks, sample_rate),
            events: Arc::from(events),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackEventKind {
    ProgramChange { program: u8 },
    NoteOff { key: u8 },
    NoteOn { key: u8, velocity: u8 },
}

impl PlaybackEventKind {
    /// 같은 sample_index 안에서 이벤트 적용 순서를 결정한다.
    fn sort_priority(&self) -> u8 {
        match self {
            Self::ProgramChange { .. } => 0,
            Self::NoteOff { .. } => 1,
            Self::NoteOn { .. } => 2,
        }
    }

    /// 정렬 안정성을 위해 노트 계열 이벤트의 키 값을 꺼낸다.
    fn note_key(&self) -> u8 {
        match self {
            Self::ProgramChange { .. } => 0,
            Self::NoteOff { key } | Self::NoteOn { key, .. } => *key,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackEvent {
    pub sample_index: usize,
    pub channel: u8,
    pub kind: PlaybackEventKind,
}

#[derive(Clone, Debug, Default)]
pub struct PlaybackData {
    pub song_length_samples: usize,
    pub events: Arc<[PlaybackEvent]>,
}

/// tempo map을 순회하며 tick 누적 시간을 초 단위로 적분한다.
pub fn seconds_for_tick_with_tempo(ppq: u16, tempo_changes: &[TempoChange], tick: u64) -> f64 {
    if ppq == 0 || tick == 0 {
        return 0.0;
    }

    let mut last_tick = 0_u64;
    let mut current_tempo = tempo_changes
        .first()
        .map(|change| change.micros_per_quarter)
        .unwrap_or(500_000);
    let mut seconds = 0.0;

    for change in tempo_changes.iter().skip(1) {
        if change.tick >= tick {
            break;
        }

        let ticks_passed = change.tick.saturating_sub(last_tick);
        seconds += ticks_passed as f64 * current_tempo as f64 / 1_000_000.0 / ppq as f64;
        last_tick = change.tick;
        current_tempo = change.micros_per_quarter;
    }

    let ticks_passed = tick.saturating_sub(last_tick);
    seconds + ticks_passed as f64 * current_tempo as f64 / 1_000_000.0 / ppq as f64
}

/// 초 단위 시간을 tempo map 기준의 tick 위치로 역산한다.
pub fn tick_for_seconds_with_tempo(ppq: u16, tempo_changes: &[TempoChange], seconds: f64) -> u64 {
    if ppq == 0 || seconds <= 0.0 {
        return 0;
    }

    let mut last_tick = 0_u64;
    let mut current_tempo = tempo_changes
        .first()
        .map(|change| change.micros_per_quarter)
        .unwrap_or(500_000);
    let mut remaining_seconds = seconds;

    for change in tempo_changes.iter().skip(1) {
        let ticks_in_segment = change.tick.saturating_sub(last_tick);
        let seconds_per_tick = current_tempo as f64 / 1_000_000.0 / ppq as f64;
        let seconds_in_segment = ticks_in_segment as f64 * seconds_per_tick;
        if remaining_seconds < seconds_in_segment {
            return last_tick + (remaining_seconds / seconds_per_tick).floor() as u64;
        }

        remaining_seconds -= seconds_in_segment;
        last_tick = change.tick;
        current_tempo = change.micros_per_quarter;
    }

    let seconds_per_tick = current_tempo as f64 / 1_000_000.0 / ppq as f64;
    last_tick + (remaining_seconds / seconds_per_tick).floor() as u64
}

#[cfg(test)]
mod tests {
    use super::{
        NoteSpan, PlaybackEventKind, ProgramChangeEvent, Song, SongMeta, TempoChange, TrackModel,
    };

    #[test]
    fn converts_ticks_with_tempo_changes_in_both_directions() {
        let song = Song {
            ppq: 480,
            total_ticks: 960,
            total_seconds: 1.5,
            meta: SongMeta::default(),
            tempo_changes: vec![
                TempoChange {
                    tick: 0,
                    micros_per_quarter: 500_000,
                },
                TempoChange {
                    tick: 480,
                    micros_per_quarter: 1_000_000,
                },
            ],
            tracks: vec![],
        };

        assert!((song.seconds_for_tick(480) - 0.5).abs() < f64::EPSILON);
        assert!((song.seconds_for_tick(960) - 1.5).abs() < f64::EPSILON);
        assert_eq!(song.tick_for_sample(44_100, 44_100), 720);
    }

    #[test]
    fn builds_playback_events_in_stable_order() {
        let song = Song {
            ppq: 480,
            total_ticks: 480,
            total_seconds: 0.5,
            meta: SongMeta::default(),
            tempo_changes: vec![TempoChange {
                tick: 0,
                micros_per_quarter: 500_000,
            }],
            tracks: vec![TrackModel {
                track_index: 0,
                name: "Track 1".to_string(),
                channel: 0,
                program: 10,
                instrument_name: String::new(),
                program_changes: vec![ProgramChangeEvent {
                    tick: 0,
                    program: 10,
                }],
                note_spans: vec![NoteSpan {
                    key: 60,
                    start_tick: 0,
                    end_tick: 480,
                    velocity: 100,
                }],
                is_muted: false,
                volume: 100.0,
            }],
        };

        let playback = song.playback_data(48_000, None);
        assert_eq!(playback.song_length_samples, 24_000);
        assert!(matches!(
            playback.events[0].kind,
            PlaybackEventKind::ProgramChange { program: 10 }
        ));
        assert!(matches!(
            playback.events[1].kind,
            PlaybackEventKind::NoteOn {
                key: 60,
                velocity: 100
            }
        ));
        assert!(matches!(
            playback.events[2].kind,
            PlaybackEventKind::NoteOff { key: 60 }
        ));
    }
}
