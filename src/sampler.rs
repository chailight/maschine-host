use crate::model::{
    nominal_latency_ms, Destination, HostConfig, PadMode, SampleZone, SamplerCommandKind,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Voice {
    id: u64,
    samples: Arc<Vec<f32>>,
    position: f64,
    rate: f64,
    choke: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderStats {
    pub clips: u64,
    pub active_voices: usize,
}

pub struct Mixer {
    samples: HashMap<String, Arc<Vec<f32>>>,
    voices: Vec<Voice>,
    polyphony: usize,
    sample_rate: u32,
    buffer_frames: u32,
}

impl Mixer {
    pub fn empty(polyphony: usize, sample_rate: u32, buffer_frames: u32) -> Self {
        Self {
            samples: HashMap::new(),
            voices: Vec::new(),
            polyphony: polyphony.max(1),
            sample_rate,
            buffer_frames,
        }
    }

    pub fn insert_sample(&mut self, path: impl Into<String>, samples: Vec<f32>) {
        self.samples.insert(path.into(), Arc::new(samples));
    }

    pub fn voice_count(&self) -> usize {
        self.voices.len()
    }

    pub fn nominal_latency_ms(&self) -> f64 {
        nominal_latency_ms(self.buffer_frames, self.sample_rate)
    }

    pub fn apply(&mut self, command: &SamplerCommandKind) -> Result<(), String> {
        match command {
            SamplerCommandKind::Trigger {
                voice_id,
                path,
                rate,
                choke,
            } => {
                let Some(samples) = self.samples.get(path).cloned() else {
                    return Err(format!("sample '{path}' is not preloaded"));
                };
                if let Some(group) = choke {
                    self.voices.retain(|voice| voice.choke != Some(*group));
                }
                while self.voices.len() >= self.polyphony {
                    self.voices.remove(0);
                }
                self.voices.push(Voice {
                    id: *voice_id,
                    samples,
                    position: 0.0,
                    rate: *rate,
                    choke: *choke,
                });
                Ok(())
            }
            SamplerCommandKind::Stop { voice_id } => {
                self.voices.retain(|voice| voice.id != *voice_id);
                Ok(())
            }
            SamplerCommandKind::StopAll => {
                self.voices.clear();
                Ok(())
            }
        }
    }

    pub fn render(&mut self, output: &mut [f32]) -> RenderStats {
        let mut clips = 0;
        for frame in output.iter_mut() {
            let mut mix = 0.0;
            for voice in &mut self.voices {
                let length = voice.samples.len();
                if length == 0 {
                    continue;
                }
                let index = voice.position.floor() as usize;
                if index >= length {
                    continue;
                }
                let next = voice.samples.get(index + 1).copied().unwrap_or(0.0);
                let fraction = (voice.position - index as f64) as f32;
                mix += voice.samples[index] + (next - voice.samples[index]) * fraction;
                voice.position += voice.rate;
            }
            if mix > 1.0 {
                mix = 1.0;
                clips += 1;
            } else if mix < -1.0 {
                mix = -1.0;
                clips += 1;
            }
            *frame = mix;
        }
        self.voices
            .retain(|voice| voice.position < voice.samples.len() as f64);
        RenderStats {
            clips,
            active_voices: self.voices.len(),
        }
    }
}

pub fn select_zone(zones: &[SampleZone], note: u8) -> Option<&SampleZone> {
    let mut best: Option<&SampleZone> = None;
    for zone in zones {
        if note < zone.lo || note > zone.hi {
            continue;
        }
        best = Some(match best {
            None => zone,
            Some(current) => closer(current, zone, note),
        });
    }
    if best.is_some() {
        return best;
    }
    zones
        .iter()
        .reduce(|current, zone| closer(current, zone, note))
}

fn closer<'a>(left: &'a SampleZone, right: &'a SampleZone, note: u8) -> &'a SampleZone {
    let left_distance = (i16::from(left.root) - i16::from(note)).abs();
    let right_distance = (i16::from(right.root) - i16::from(note)).abs();
    if right_distance < left_distance {
        right
    } else {
        left
    }
}

pub fn preload(config: &HostConfig) -> Result<Mixer, Vec<String>> {
    let mut mixer = Mixer::empty(config.polyphony, config.sample_rate, config.buffer_frames);
    let mut errors = Vec::new();
    let mut paths = Vec::new();
    for group in &config.groups {
        if !matches!(group.destination, Destination::Sampler { .. }) {
            continue;
        }
        match group.mode {
            PadMode::OneShot => {
                for sample in group.samples.iter().flatten() {
                    paths.push(sample.path.clone());
                }
            }
            PadMode::Chromatic | PadMode::Scale => {
                for zone in &group.zones {
                    paths.push(zone.path.clone());
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    for path in paths {
        match load_wav(Path::new(&path)) {
            Ok(samples) => mixer.insert_sample(path, samples),
            Err(message) => errors.push(message),
        }
    }
    if errors.is_empty() {
        Ok(mixer)
    } else {
        Err(errors)
    }
}

pub fn load_wav(path: &Path) -> Result<Vec<f32>, String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut reader = hound::WavReader::new(BufReader::new(file))
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let spec = reader.spec();
    if spec.channels == 0 || spec.sample_rate == 0 {
        return Err(format!("{}: invalid WAV header", path.display()));
    }
    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(format!(
            "{}: only 16-bit integer PCM WAV is supported (found {}-bit {:?})",
            path.display(),
            spec.bits_per_sample,
            spec.sample_format
        ));
    }
    let channels = spec.channels as usize;
    let samples: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    let mut mono = Vec::with_capacity(samples.len() / channels);
    for frame in samples.chunks(channels) {
        let sum: i32 = frame.iter().map(|sample| i32::from(*sample)).sum();
        let mean = sum / channels as i32;
        mono.push(mean as f32 / i16::MAX as f32);
    }
    Ok(mono)
}

pub fn write_pcm16(path: &Path, sample_rate: u32, samples: &[f32]) -> Result<(), String> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    for sample in samples {
        let scaled = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        writer
            .write_sample(scaled)
            .map_err(|error| format!("{}: {error}", path.display()))?;
    }
    writer
        .finalize()
        .map_err(|error| format!("{}: {error}", path.display()))?;
    Ok(())
}

pub trait AudioSink {
    fn write(&mut self, samples: &[f32]) -> Result<(), String>;
}

pub struct FailingSink;

impl AudioSink for FailingSink {
    fn write(&mut self, _samples: &[f32]) -> Result<(), String> {
        Err("audio device failed".to_string())
    }
}

pub fn pump(
    mixer: &mut Mixer,
    sink: &mut dyn AudioSink,
    buffer: &mut [f32],
) -> Result<RenderStats, String> {
    let stats = mixer.render(buffer);
    sink.write(buffer)?;
    Ok(stats)
}

#[cfg(feature = "cpal-output")]
pub fn probe_default_output() -> Result<String, String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "no default output device".to_string())?;
    let name = device
        .name()
        .map_err(|error| format!("device name: {error}"))?;
    let config = device
        .default_output_config()
        .map_err(|error| format!("default config for {name}: {error}"))?;
    Ok(format!(
        "{name} sample_rate={} channels={:?} sample_format={:?}",
        config.sample_rate().0,
        config.channels(),
        config.sample_format()
    ))
}

#[cfg(not(feature = "cpal-output"))]
pub fn probe_default_output() -> Result<String, String> {
    Err("cpal output probe is disabled; rebuild with --features cpal-output".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_wav(name: &str, samples: &[f32]) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("maschine2-{name}-{nanos}.wav"));
        write_pcm16(&path, 48_000, samples).unwrap();
        path
    }

    #[test]
    fn nearest_zone_wins_inside_its_range() {
        let zones = vec![
            SampleZone {
                path: "low".into(),
                root: 36,
                lo: 0,
                hi: 47,
            },
            SampleZone {
                path: "mid".into(),
                root: 60,
                lo: 48,
                hi: 72,
            },
        ];
        assert_eq!(select_zone(&zones, 50).unwrap().path, "mid");
        assert_eq!(select_zone(&zones, 20).unwrap().path, "low");
    }

    #[test]
    fn one_shot_finishes_and_choke_replaces_the_previous_voice() {
        let mut mixer = Mixer::empty(4, 48_000, 8);
        mixer.insert_sample("a", vec![0.25; 4]);
        mixer.insert_sample("b", vec![0.5; 4]);
        mixer
            .apply(&SamplerCommandKind::Trigger {
                voice_id: 1,
                path: "a".into(),
                rate: 1.0,
                choke: Some(1),
            })
            .unwrap();
        mixer
            .apply(&SamplerCommandKind::Trigger {
                voice_id: 2,
                path: "b".into(),
                rate: 1.0,
                choke: Some(1),
            })
            .unwrap();
        assert_eq!(mixer.voice_count(), 1);
        let mut buffer = [0.0; 8];
        let stats = mixer.render(&mut buffer);
        assert_eq!(stats.active_voices, 0);
        assert!(buffer.iter().take(4).any(|sample| *sample > 0.4));
    }

    #[test]
    fn polyphony_drops_the_oldest_voice() {
        let mut mixer = Mixer::empty(2, 48_000, 16);
        mixer.insert_sample("a", vec![1.0; 32]);
        for id in 1..=3 {
            mixer
                .apply(&SamplerCommandKind::Trigger {
                    voice_id: id,
                    path: "a".into(),
                    rate: 1.0,
                    choke: None,
                })
                .unwrap();
        }
        assert_eq!(mixer.voice_count(), 2);
        assert!(mixer.voices.iter().all(|voice| voice.id != 1));
    }

    #[test]
    fn clipping_is_counted_and_limited() {
        let mut mixer = Mixer::empty(4, 48_000, 8);
        mixer.insert_sample("a", vec![1.0; 4]);
        for id in 1..=3 {
            mixer
                .apply(&SamplerCommandKind::Trigger {
                    voice_id: id,
                    path: "a".into(),
                    rate: 1.0,
                    choke: None,
                })
                .unwrap();
        }
        let mut buffer = [0.0; 2];
        let stats = mixer.render(&mut buffer);
        assert!(stats.clips > 0);
        assert!(buffer
            .iter()
            .all(|sample| *sample <= 1.0 && *sample >= -1.0));
    }

    #[test]
    fn chromatic_rate_consumes_the_sample_faster() {
        let mut mixer = Mixer::empty(1, 48_000, 8);
        mixer.insert_sample("a", vec![0.2; 8]);
        mixer
            .apply(&SamplerCommandKind::Trigger {
                voice_id: 1,
                path: "a".into(),
                rate: 2.0,
                choke: None,
            })
            .unwrap();
        let mut buffer = [0.0; 4];
        mixer.render(&mut buffer);
        assert_eq!(mixer.voice_count(), 0);
    }

    #[test]
    fn nominal_latency_is_buffer_time_not_a_measured_round_trip() {
        let mixer = Mixer::empty(1, 48_000, 256);
        let latency = mixer.nominal_latency_ms();
        assert!((latency - (256.0 * 1000.0 / 48_000.0)).abs() < 0.001);
    }

    #[test]
    fn device_failure_is_recoverable() {
        let mut mixer = Mixer::empty(1, 48_000, 8);
        mixer.insert_sample("a", vec![0.1; 4]);
        mixer
            .apply(&SamplerCommandKind::Trigger {
                voice_id: 1,
                path: "a".into(),
                rate: 1.0,
                choke: None,
            })
            .unwrap();
        let mut buffer = [0.0; 4];
        let error = pump(&mut mixer, &mut FailingSink, &mut buffer).unwrap_err();
        assert!(error.contains("audio device"));
    }

    #[test]
    fn wav_round_trip_accepts_pcm16_and_rejects_float() {
        let path = temp_wav("pcm", &[0.0, 0.5, -0.5]);
        let loaded = load_wav(&path).unwrap();
        assert_eq!(loaded.len(), 3);
        assert!(loaded[1] > 0.4);
        let float_path = path.with_file_name("maschine2-float.wav");
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&float_path, spec).unwrap();
        writer.write_sample(0.25_f32).unwrap();
        writer.finalize().unwrap();
        let error = load_wav(&float_path).unwrap_err();
        assert!(error.contains("16-bit integer PCM"));
        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(float_path);
    }
}
