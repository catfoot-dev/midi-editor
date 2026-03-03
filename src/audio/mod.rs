use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rustysynth::{SoundFont, Synthesizer, SynthesizerSettings};
use std::{fs::File, sync::{Arc, Mutex}};

use crate::audio::state::SharedAudioState;

pub mod midi;
pub mod midi_struct;
pub mod state;

pub struct Audio {
    _stream: cpal::Stream,
    pub sample_rate: i32,
}

impl Audio {
    // 생성자에서 공유 상태(state)를 받습니다.
    pub fn new(state: Arc<Mutex<SharedAudioState>>) -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        let config: cpal::StreamConfig = device.default_output_config().unwrap().config().into();
        let sample_rate = config.sample_rate.0 as i32;

        // 신디사이저 초기화 (SoundFont 로드)
        let mut sf2 = File::open("assets/GeneralUser-GS.sf2").unwrap(); // 파일 필요!
        let sf = Arc::new(SoundFont::new(&mut sf2).unwrap());
        let settings = SynthesizerSettings::new(sample_rate);
        let mut synthesizer = Synthesizer::new(&sf, &settings).unwrap();

        // 오디오 콜백 (여기가 백그라운드 스레드)
        let stream = device.build_output_stream(
            &config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // 공유 상태 잠그기 (Lock)
                if let Ok(mut shared) = state.lock() {
                    process_audio_block(data, &mut synthesizer, &mut shared);
                }
            },
            |err| eprintln!("Audio Error: {}", err),
            None,
        ).unwrap();

        stream.play().unwrap();

        Self {
            _stream: stream,
            sample_rate,
        }
    }
}

// 실제 오디오 처리 로직 (콜백 내부)
fn process_audio_block(
    buffer: &mut [f32],
    synth: &mut Synthesizer,
    shared: &mut SharedAudioState,
) {
    let frame_count = buffer.len() / 2; // 스테레오라 2로 나눔

    // 재생 중이 아니면 소리 끄고 리턴 (무음 처리)
    // 버퍼를 0으로 채움 (Mute)
    if !shared.is_playing {
        buffer.iter_mut().for_each(|x| *x = 0.0);
        return; 
    }

    for (channel, instrument) in shared.instruments.iter() {
        synth.process_midi_message(*channel as i32, 0xC0, *instrument, 0);
    }

    let current_time = shared.playback_cursor;
    let next_time = current_time + frame_count;
    for note in shared.notes.iter() {
        // 현재 버퍼 시간 내에 노트 시작점이 있는지 확인
        if note.start_sample >= current_time && note.start_sample < next_time {
            synth.note_on(note.channel, note.key, note.velocity);
        }
    }

    // 신디사이저 렌더링
    let mut left = vec![0.0; frame_count];
    let mut right = vec![0.0; frame_count];
    synth.render(&mut left, &mut right);

    // 버퍼 채우기 & 시간 업데이트
    for (i, frame) in buffer.chunks_mut(2).enumerate() {
        frame[0] = left[i];
        frame[1] = right[i];
    }
    
    // 재생 커서 이동
    shared.playback_cursor += frame_count;
}