use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;
use std::sync::Arc;
use rodio::decoder::Decoder;
use rodio::Source;
use show_message::OkOrShow;
use heck::SnakeCase;

#[repr(C)]
#[derive(Deserialize, Clone, Copy, Debug, EnumIterator)]
pub enum Sound {
    BongoH,
    BongoL,
    Clave,
    Conga,
}

// 44100 Hz mono buffer
struct SoundBuffer {
    samples: Arc<Vec<i16>>,
}

impl SoundBuffer {
    fn new(sound: Decoder<Cursor<Vec<u8>>>) -> Result<Self, String> {
        if sound.sample_rate() != 44100 {
            return Err("invalid samples rate: must be 44100 Hz".into());
        }
        if sound.channels() != 1 {
            return Err("invalid channels: must be mono".into());
        }

        Ok(SoundBuffer {
            samples: Arc::new(sound.collect::<Vec<_>>()),
        })
    }

    fn source(&self) -> SoundSource {
        SoundSource {
            samples: self.samples.clone(),
            cursor: 0,
        }
    }

    #[allow(unused)]
    fn infinite_source(&self) -> InfiniteSoundSource {
        InfiniteSoundSource {
            samples: self.samples.clone(),
            cursor: 0,
            len: self.samples.len(),
        }
    }
}

// infinite sound soure from a 44100 Hz mono buffer
#[allow(unused)]
struct InfiniteSoundSource {
    samples: Arc<Vec<i16>>,
    cursor: usize,
    len: usize,
}

impl Iterator for InfiniteSoundSource {
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.samples.get(self.cursor).cloned();
        self.cursor = (self.cursor + 1) % self.len;
        sample
    }
}

impl Source for InfiniteSoundSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        44100
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// sound soure from a 44100 Hz mono buffer
struct SoundSource {
    samples: Arc<Vec<i16>>,
    cursor: usize,
}

impl Iterator for SoundSource {
    type Item = i16;
    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.samples.get(self.cursor).cloned();
        self.cursor += 1;
        sample
    }
}

impl Source for SoundSource {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }
    fn channels(&self) -> u16 {
        1
    }
    fn sample_rate(&self) -> u32 {
        44100
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

lazy_static! {
    static ref SOUND_BUFFERS: Vec<SoundBuffer> = {
        let sound_filenames = Sound::iter_variants()
            .map(|variant| {
                let name = format!("{:?}", variant);
                format!("data/sounds/{}.ogg", name.to_snake_case())
            })
            .collect::<Vec<_>>();

        let mut sound_files = sound_filenames.iter()
            .map(|s| {
                let mut buffer = vec![];
                let mut file = File::open(s)
                    .ok_or_show(|e| format!("Failed to open sound {}: {}", s, e));
                file.read_to_end(&mut buffer)
                    .ok_or_show(|e| format!("Failed to read sound {}: {}", s, e));
                Cursor::new(buffer)
            })
            .collect::<Vec<_>>();

        let mut sound_buffers = vec![];
        for (file, filename) in sound_files.drain(..).zip(sound_filenames.iter()) {
            let sound = Decoder::new(file)
                .ok_or_show(|e| format!("Failed to decode sound {}: {}", filename, e));

            let sound = SoundBuffer::new(sound)
                .ok_or_show(|e| format!("Invalid sound: {}: {}", filename, e));

            sound_buffers.push(sound);
        }
        sound_buffers
    };
}

pub struct Audio {
    mixer: Arc<::dynamic_mixer::DynamicMixerController<i16>>,

    position: ::na::Vector2<f32>,

    unit: f32,
    clamp: ::util::ClampFunction,
}

impl Audio {
    pub fn init(conf: &::resource::Conf, save: &::resource::Save) -> Self {
        let device = ::rodio::default_output_device().unwrap();

        let (mixer_ctrl, mixer) = ::dynamic_mixer::mixer(2, 44100);

        let sink = ::rodio::Sink::new(&device);
        sink.append(mixer.mix(::rodio::source::Zero::<i16>::new(2, 44100)));
        sink.detach();

        Audio {
            position: ::na::Vector2::new(0.0, 0.0),
            unit: conf.audio_unit,
            clamp: ::util::ClampFunction {
                min_t: conf.audio_clamp_start,
                max_t: conf.audio_clamp_end,
                min_value: save.audio_volume,
                max_value: 0.0,
            },
            mixer: mixer_ctrl,
        }
    }

    pub fn play(&self, sound: Sound, position: ::na::Vector2<f32>) {
        let position = position * self.unit;
        let distance = position - self.position;
        let volume = self.clamp.compute(distance.norm());
        let pan = ((distance[0].min(1.0).max(-1.0)+1.0)/2.0).min(1.0).max(0.0);

        let left_volume = (1.0-pan).sqrt()*volume;
        let right_volume = pan.sqrt()*volume;

        let source = ::rodio::source::ChannelVolume::new(SOUND_BUFFERS[sound as usize].source(), vec![left_volume, right_volume]);
        self.mixer.add(source);
    }

    pub fn update(&mut self, position: Option<::na::Vector2<f32>>, save: &::resource::Save) {
        self.clamp.min_value = save.audio_volume;
        if let Some(position) = position {
            self.position = position*self.unit;
        }
    }
}
