use std::sync::Arc;

use crate::audio::midi_struct::{PlaybackData, PlaybackEvent};

#[derive(Debug)]
pub struct SharedAudioState {
    pub is_playing: bool,
    pub repeat_enabled: bool,
    pub playback_cursor_samples: usize,
    pub song_length_samples: usize,
    pub event_cursor: usize,
    pub playback_events: Arc<[PlaybackEvent]>,
    pub pending_reset: bool,
    pub revision: u64,
}

impl Default for SharedAudioState {
    fn default() -> Self {
        Self {
            is_playing: false,
            repeat_enabled: false,
            playback_cursor_samples: 0,
            song_length_samples: 0,
            event_cursor: 0,
            playback_events: Arc::from(Vec::<PlaybackEvent>::new()),
            pending_reset: false,
            revision: 0,
        }
    }
}

impl SharedAudioState {
    /// 새 재생 타임라인을 반영하고 필요하면 커서를 처음으로 되돌린다.
    pub fn set_playback_data(&mut self, playback_data: PlaybackData, reset_cursor: bool) {
        self.playback_events = playback_data.events;
        self.song_length_samples = playback_data.song_length_samples;
        if reset_cursor {
            self.playback_cursor_samples = 0;
        } else {
            self.playback_cursor_samples =
                self.playback_cursor_samples.min(self.song_length_samples);
        }
        // seek 이후 바로 이어서 재생할 수 있도록 이벤트 커서를 현재 샘플 위치에 맞춘다.
        self.event_cursor = self.find_event_cursor(self.playback_cursor_samples);
        self.pending_reset = true;
        self.revision = self.revision.wrapping_add(1);
    }

    /// 곡을 닫을 때 재생 상태를 완전히 초기화한다.
    pub fn clear_playback(&mut self) {
        self.is_playing = false;
        self.repeat_enabled = false;
        self.playback_cursor_samples = 0;
        self.song_length_samples = 0;
        self.event_cursor = 0;
        self.playback_events = Arc::from(Vec::<PlaybackEvent>::new());
        self.pending_reset = true;
        self.revision = self.revision.wrapping_add(1);
    }

    /// 반복 재생 여부만 바꾸고 오디오 스레드가 변경을 감지할 수 있도록 revision을 올린다.
    pub fn set_repeat_enabled(&mut self, enabled: bool) {
        self.repeat_enabled = enabled;
        self.revision = self.revision.wrapping_add(1);
    }

    /// 재생/일시정지 전환 시 현재 커서 기준으로 신디사이저 상태를 다시 만들 수 있게 준비한다.
    pub fn set_playing(&mut self, is_playing: bool) {
        if self.is_playing == is_playing {
            return;
        }

        self.is_playing = is_playing;
        self.event_cursor = self.find_event_cursor(self.playback_cursor_samples);
        self.pending_reset = true;
        self.revision = self.revision.wrapping_add(1);
    }

    /// 슬라이더 탐색 등으로 샘플 커서를 옮길 때 이벤트 커서도 함께 재계산한다.
    pub fn seek_samples(&mut self, sample_index: usize) {
        self.playback_cursor_samples = sample_index.min(self.song_length_samples);
        self.event_cursor = self.find_event_cursor(self.playback_cursor_samples);
        self.pending_reset = true;
        self.revision = self.revision.wrapping_add(1);
    }

    /// 현재 샘플 위치 이전의 이벤트 개수를 계산해 재생 시작 지점을 찾는다.
    pub fn find_event_cursor(&self, sample_index: usize) -> usize {
        self.playback_events
            .partition_point(|event| event.sample_index < sample_index)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::audio::midi_struct::{PlaybackData, PlaybackEvent, PlaybackEventKind};

    use super::SharedAudioState;

    #[test]
    fn seeking_recomputes_event_cursor() {
        let mut state = SharedAudioState::default();
        state.set_playback_data(
            PlaybackData {
                song_length_samples: 10_000,
                events: Arc::from(vec![
                    PlaybackEvent {
                        sample_index: 0,
                        channel: 0,
                        kind: PlaybackEventKind::ProgramChange { program: 0 },
                    },
                    PlaybackEvent {
                        sample_index: 5_000,
                        channel: 0,
                        kind: PlaybackEventKind::NoteOn {
                            key: 60,
                            velocity: 100,
                        },
                    },
                    PlaybackEvent {
                        sample_index: 7_500,
                        channel: 0,
                        kind: PlaybackEventKind::NoteOff { key: 60 },
                    },
                ]),
            },
            true,
        );

        state.seek_samples(6_000);
        assert_eq!(state.event_cursor, 2);
    }
}
