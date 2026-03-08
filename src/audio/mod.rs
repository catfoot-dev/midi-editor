use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SampleFormat, SizedSample,
};
use rustysynth::{SoundFont, SoundFontError, Synthesizer, SynthesizerError, SynthesizerSettings};
use std::{
    fmt,
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::audio::{
    midi_struct::{PlaybackEvent, PlaybackEventKind},
    state::SharedAudioState,
};

pub mod midi;
pub mod midi_struct;
pub mod state;

pub struct Audio {
    _stream: cpal::Stream,
    pub sample_rate: u32,
}

#[derive(Debug)]
pub enum AudioInitError {
    NoOutputDevice,
    DefaultOutputConfig(cpal::DefaultStreamConfigError),
    BuildStream(cpal::BuildStreamError),
    PlayStream(cpal::PlayStreamError),
    OpenSoundFont(std::io::Error),
    ParseSoundFont(SoundFontError),
    CreateSynth(SynthesizerError),
    UnsupportedSampleFormat(SampleFormat),
}

impl fmt::Display for AudioInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoOutputDevice => write!(f, "no default audio output device is available"),
            Self::DefaultOutputConfig(error) => {
                write!(f, "failed to get default output configuration: {error}")
            }
            Self::BuildStream(error) => write!(f, "failed to build output stream: {error}"),
            Self::PlayStream(error) => write!(f, "failed to start output stream: {error}"),
            Self::OpenSoundFont(error) => write!(f, "failed to open bundled SoundFont: {error}"),
            Self::ParseSoundFont(error) => write!(f, "failed to parse bundled SoundFont: {error}"),
            Self::CreateSynth(error) => write!(f, "failed to initialize synthesizer: {error}"),
            Self::UnsupportedSampleFormat(sample_format) => {
                write!(f, "unsupported sample format: {sample_format}")
            }
        }
    }
}

impl std::error::Error for AudioInitError {}

impl Audio {
    /// 출력 장치와 SoundFont를 초기화하고 오디오 스트림을 시작한다.
    pub fn new(state: Arc<Mutex<SharedAudioState>>) -> Result<Self, AudioInitError> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioInitError::NoOutputDevice)?;
        let supported_config = device
            .default_output_config()
            .map_err(AudioInitError::DefaultOutputConfig)?;
        let sample_format = supported_config.sample_format();
        let config = supported_config.config();
        let sample_rate = config.sample_rate;

        // 실행 위치가 달라도 항상 프로젝트 자산을 찾을 수 있도록 고정 경로를 사용한다.
        let sound_font_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets")
            .join("GeneralUser-GS.sf2");
        let mut sound_font_file =
            File::open(sound_font_path).map_err(AudioInitError::OpenSoundFont)?;
        let sound_font = Arc::new(
            SoundFont::new(&mut sound_font_file).map_err(AudioInitError::ParseSoundFont)?,
        );
        let settings = SynthesizerSettings::new(sample_rate as i32);
        let synthesizer =
            Synthesizer::new(&sound_font, &settings).map_err(AudioInitError::CreateSynth)?;

        let stream = match sample_format {
            SampleFormat::I8 => build_output_stream::<i8>(&device, &config, state, synthesizer),
            SampleFormat::I16 => build_output_stream::<i16>(&device, &config, state, synthesizer),
            SampleFormat::I32 => build_output_stream::<i32>(&device, &config, state, synthesizer),
            SampleFormat::I64 => build_output_stream::<i64>(&device, &config, state, synthesizer),
            SampleFormat::U8 => build_output_stream::<u8>(&device, &config, state, synthesizer),
            SampleFormat::U16 => build_output_stream::<u16>(&device, &config, state, synthesizer),
            SampleFormat::U32 => build_output_stream::<u32>(&device, &config, state, synthesizer),
            SampleFormat::U64 => build_output_stream::<u64>(&device, &config, state, synthesizer),
            SampleFormat::F32 => build_output_stream::<f32>(&device, &config, state, synthesizer),
            SampleFormat::F64 => build_output_stream::<f64>(&device, &config, state, synthesizer),
            unsupported => return Err(AudioInitError::UnsupportedSampleFormat(unsupported)),
        }
        .map_err(AudioInitError::BuildStream)?;

        stream.play().map_err(AudioInitError::PlayStream)?;

        Ok(Self {
            _stream: stream,
            sample_rate,
        })
    }
}

fn build_output_stream<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    state: Arc<Mutex<SharedAudioState>>,
    mut synthesizer: Synthesizer,
) -> Result<cpal::Stream, cpal::BuildStreamError>
where
    T: SizedSample + FromSample<f32>,
{
    // 콜백이 매 블록마다 할당하지 않도록 렌더 버퍼를 클로저 안에 보관한다.
    let channels = config.channels as usize;
    let mut left = Vec::new();
    let mut right = Vec::new();

    device.build_output_stream(
        config,
        move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
            render_output_block(data, channels, &state, &mut synthesizer, &mut left, &mut right)
        },
        |error| eprintln!("Audio Error: {error}"),
        None,
    )
}

fn render_output_block<T>(
    output: &mut [T],
    channels: usize,
    state: &Arc<Mutex<SharedAudioState>>,
    synthesizer: &mut Synthesizer,
    left: &mut Vec<f32>,
    right: &mut Vec<f32>,
) where
    T: Sample + FromSample<f32>,
{
    // 장치 포맷에 맞는 출력 버퍼를 채우기 전에 내부 상태를 스냅샷으로 꺼낸다.
    if channels == 0 {
        return;
    }

    let frame_count = output.len() / channels;
    if frame_count == 0 {
        return;
    }

    resize_render_buffers(left, right, frame_count);

    let snapshot = {
        let Ok(shared_state) = state.lock() else {
            fill_silence(output);
            return;
        };

        AudioSnapshot {
            is_playing: shared_state.is_playing,
            repeat_enabled: shared_state.repeat_enabled,
            playback_cursor_samples: shared_state.playback_cursor_samples,
            song_length_samples: shared_state.song_length_samples,
            event_cursor: shared_state.event_cursor,
            playback_events: Arc::clone(&shared_state.playback_events),
            pending_reset: shared_state.pending_reset,
            revision: shared_state.revision,
        }
    };

    if snapshot.pending_reset {
        // seek/play 토글 뒤에는 현재 커서 이전 이벤트를 다시 적용해 신디사이저 상태를 복원한다.
        synthesizer.reset();
        prime_synth(
            synthesizer,
            &snapshot.playback_events,
            snapshot.event_cursor,
        );
    }

    if !snapshot.is_playing
        || snapshot.song_length_samples == 0
        || snapshot.playback_events.is_empty()
    {
        fill_silence(output);
        if let Ok(mut shared_state) = state.lock()
            && shared_state.revision == snapshot.revision
        {
            shared_state.pending_reset = false;
        }
        return;
    }

    let mut playback_cursor_samples = snapshot.playback_cursor_samples;
    let mut event_cursor = snapshot.event_cursor;
    let mut still_playing = true;

    if playback_cursor_samples >= snapshot.song_length_samples {
        if snapshot.repeat_enabled {
            synthesizer.reset();
            playback_cursor_samples = 0;
            event_cursor = 0;
        } else {
            fill_silence(output);
            still_playing = false;
        }
    }

    if still_playing {
        let next_cursor = playback_cursor_samples.saturating_add(frame_count);
        // 이번 블록 범위에 들어오는 이벤트만 적용하고 렌더는 한 번만 수행한다.
        while event_cursor < snapshot.playback_events.len() {
            let event = &snapshot.playback_events[event_cursor];
            if event.sample_index >= next_cursor {
                break;
            }
            apply_event(synthesizer, event);
            event_cursor += 1;
        }

        synthesizer.render(left, right);
        write_output(output, channels, left, right);
        playback_cursor_samples = next_cursor;

        if playback_cursor_samples >= snapshot.song_length_samples {
            if snapshot.repeat_enabled {
                synthesizer.reset();
                playback_cursor_samples = 0;
                event_cursor = 0;
            } else {
                still_playing = false;
            }
        }
    }

    if let Ok(mut shared_state) = state.lock()
        && shared_state.revision == snapshot.revision
    {
        shared_state.playback_cursor_samples = playback_cursor_samples;
        shared_state.event_cursor = event_cursor;
        shared_state.is_playing = still_playing;
        shared_state.pending_reset = !still_playing;
    }
}

fn resize_render_buffers(left: &mut Vec<f32>, right: &mut Vec<f32>, frame_count: usize) {
    // CPAL 콜백 길이는 장치에 따라 달라질 수 있으므로 필요할 때만 크기를 조정한다.
    if left.len() != frame_count {
        left.resize(frame_count, 0.0);
    } else {
        left.fill(0.0);
    }

    if right.len() != frame_count {
        right.resize(frame_count, 0.0);
    } else {
        right.fill(0.0);
    }
}

fn prime_synth(synthesizer: &mut Synthesizer, events: &[PlaybackEvent], event_cursor: usize) {
    // 현재 커서보다 앞선 이벤트를 다시 재생해 seek 이후의 음 상태를 맞춘다.
    for event in events.iter().take(event_cursor) {
        apply_event(synthesizer, event);
    }
}

fn apply_event(synthesizer: &mut Synthesizer, event: &PlaybackEvent) {
    // 오디오 스레드는 사전에 정규화된 PlaybackEvent만 이해하면 된다.
    match event.kind {
        PlaybackEventKind::ProgramChange { program } => {
            synthesizer.process_midi_message(event.channel as i32, 0xC0, program as i32, 0);
        }
        PlaybackEventKind::NoteOn { key, velocity } => {
            synthesizer.note_on(event.channel as i32, key as i32, velocity as i32);
        }
        PlaybackEventKind::NoteOff { key } => {
            synthesizer.note_off(event.channel as i32, key as i32);
        }
    }
}

fn write_output<T>(output: &mut [T], channels: usize, left: &[f32], right: &[f32])
where
    T: Sample + FromSample<f32>,
{
    // 스테레오 렌더 결과를 장치 채널 수에 맞게 복사한다.
    for (frame_index, frame) in output.chunks_mut(channels).enumerate() {
        let left_sample = left[frame_index];
        let right_sample = right[frame_index];
        let mono_sample = (left_sample + right_sample) * 0.5;

        for (channel_index, sample) in frame.iter_mut().enumerate() {
            let value = if channels == 1 {
                mono_sample
            } else if channel_index % 2 == 0 {
                left_sample
            } else {
                right_sample
            };
            *sample = T::from_sample(value);
        }
    }
}

fn fill_silence<T>(output: &mut [T])
where
    T: Sample + FromSample<f32>,
{
    // 재생 중이 아닐 때는 이전 버퍼 잔향이 남지 않도록 즉시 무음으로 덮는다.
    for sample in output {
        *sample = T::from_sample(0.0);
    }
}

struct AudioSnapshot {
    is_playing: bool,
    repeat_enabled: bool,
    playback_cursor_samples: usize,
    song_length_samples: usize,
    event_cursor: usize,
    playback_events: Arc<[PlaybackEvent]>,
    pending_reset: bool,
    revision: u64,
}
