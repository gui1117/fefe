use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::time::Duration;
use std::sync::{Arc, Mutex};
use rodio::decoder::Decoder;
use rodio::Source;
use rodio::Sample;
use show_message::OkOrShow;
use heck::SnakeCase;

const MAX_DISTANCE_SOUND: f32 = 1000.0;

#[repr(C)]
#[derive(Deserialize, Clone, Copy, Debug, EnumIterator)]
pub enum Sound {
    A,
}

/// Sounds must be 44100 Hz and stereo
pub struct AudioMix {
    spatial_source: Vec<(::rodio::source::Spatial<SoundSource>, [f32; 3])>,
    unspatial_source: Vec<SoundSource>,
    left_ear: [f32; 3],
    right_ear: [f32; 3],
    delete_indices_cache: Vec<usize>,
    effect_volume: f32,
}

impl AudioMix {
    fn new(left_ear: [f32; 3], right_ear: [f32; 3], effect_volume: f32) -> Self {
        AudioMix {
            spatial_source: vec![],
            unspatial_source: vec![],
            left_ear,
            right_ear,
            delete_indices_cache: vec![],
            effect_volume: effect_volume,
        }
    }

    fn set_listener(&mut self, left_ear: [f32; 3], right_ear: [f32; 3]) {
        self.left_ear = left_ear;
        self.right_ear = right_ear;

        for &mut (ref mut source, position) in &mut self.spatial_source {
            source.set_positions(
                position,
                left_ear,
                right_ear,
            );
        }
    }

    fn add_spatial(&mut self, sound: SoundSource, position: [f32; 3]) {
        assert!(sound.channels() == 2);
        assert!(sound.sample_rate() == 44100);
        let distance_2 = (position[0]-self.left_ear[0]).powi(2)
            + (position[1]-self.left_ear[1]).powi(2)
            + (position[2]-self.left_ear[2]).powi(2);

        if distance_2 < MAX_DISTANCE_SOUND.powi(2) {
            self.spatial_source.push((::rodio::source::Spatial::new(
                sound,
                position,
                self.left_ear,
                self.right_ear,
            ), position));
        }
    }

    fn add_unspatial(&mut self, sound: SoundSource) {
        assert!(sound.channels() == 2);
        assert!(sound.sample_rate() == 44100);
        self.unspatial_source.push(sound);
    }
}

impl Iterator for AudioMix {
    type Item = i16;

    #[inline]
    fn next(&mut self) -> Option<i16> {
        let mut next = Self::Item::zero_value();

        self.delete_indices_cache.clear();
        for (i, &mut (ref mut source, _)) in self.spatial_source.iter_mut().enumerate() {
            if let Some(sample) = source.next() {
                next = next.saturating_add(sample);
            } else {
                self.delete_indices_cache.push(i);
            }
        }
        for (i, indice) in self.delete_indices_cache.drain(..).enumerate() {
            self.spatial_source.remove(indice - i);
        }

        self.delete_indices_cache.clear();
        for (i, source) in self.unspatial_source.iter_mut().enumerate() {
            if let Some(sample) = source.next() {
                next = next.saturating_add(sample);
            } else {
                self.delete_indices_cache.push(i);
            }
        }
        for (i, indice) in self.delete_indices_cache.drain(..).enumerate() {
            self.unspatial_source.remove(indice - i);
        }

        Some(next.amplify(self.effect_volume))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

impl ExactSizeIterator for AudioMix { }

impl Source for AudioMix {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        2
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        44100
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

pub struct AudioSinkControl {
    left_ear: [f32; 3],
    right_ear: [f32; 3],
    effect_volume: f32,
    spatial_sounds_to_add: Vec<(Sound, [f32; 3])>,
    sounds_to_add: Vec<Sound>,
}

impl AudioSinkControl {
    fn new(volume: f32) -> Self {
        AudioSinkControl {
            left_ear: [0f32; 3],
            right_ear: [0f32; 3],
            effect_volume: volume,
            spatial_sounds_to_add: vec![],
            sounds_to_add: vec![],
        }
    }
}

// 44100 Hz stereo buffer
struct SoundBuffer {
    samples: Arc<Vec<i16>>,
}

impl SoundBuffer {
    fn new(sound: Decoder<Cursor<Vec<u8>>>) -> Result<Self, String> {
        if sound.sample_rate() != 44100 {
            return Err("invalid samples rate: must be 44100 Hz".into());
        }
        if sound.channels() != 2 {
            return Err("invalid channels: must be stereo".into());
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

// infinite sound soure from a 44100 Hz stereo buffer
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
        2
    }
    fn sample_rate(&self) -> u32 {
        44100
    }
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

// sound soure from a 44100 Hz stereo buffer
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
        2
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
                format!("data/{}.ogg", name.to_snake_case())
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
    audio_sink_control: Option<Arc<Mutex<AudioSinkControl>>>,
    // Used to drop sink
    _audio_sink: Option<::rodio::Sink>,
}

impl Audio {
    pub fn init(save: &::resource::Save) -> Self {
        let endpoint = ::rodio::default_output_device();
        if endpoint.is_none() {
            return Audio {
                audio_sink_control: None,
                _audio_sink: None,
            };
        }
        let endpoint = endpoint.unwrap();

        let control = Arc::new(Mutex::new(AudioSinkControl::new(save.effect_volume)));
        let audio_sink_control = Some(control.clone());

        let source = AudioMix::new([0f32; 3], [0f32; 3], save.effect_volume)
            .periodic_access(
                Duration::from_millis(10),
                move |audio_mix| {
                    let mut control = control.lock().unwrap();

                    audio_mix.effect_volume = control.effect_volume;

                    audio_mix.set_listener(control.left_ear, control.right_ear);

                    for (sound, position) in control.spatial_sounds_to_add.drain(..) {
                        audio_mix.add_spatial(SOUND_BUFFERS[sound as usize].source(), position);
                    }

                    for sound in control.sounds_to_add.drain(..) {
                        audio_mix.add_unspatial(SOUND_BUFFERS[sound as usize].source());
                    }
                }
            );

        let audio_sink = ::rodio::Sink::new(&endpoint);
        audio_sink.append(source);

        Audio {
            _audio_sink: Some(audio_sink),
            audio_sink_control,
        }
    }

    #[allow(unused)]
    pub fn play_unspatial(&self, sound: Sound) {
        if let Some(ref control) = self.audio_sink_control {
            let mut control = control.lock().unwrap();
            control.sounds_to_add.push(sound);
        }
    }

    pub fn play(&self, sound: Sound, position: [f32; 2]) {
        let position = [position[0], position[1], 0.0];
        if let Some(ref control) = self.audio_sink_control {
            let mut control = control.lock().unwrap();
            control.spatial_sounds_to_add.push((sound, position));
        }
    }

    pub fn update(&mut self, position: ::na::Vector2<f32>, z: f32, ear_distance: f32, effect_volume: f32) {
        if let Some(ref control) = self.audio_sink_control {
            let local_left_ear = ::na::Point3::new(-ear_distance/2.0, 0.0, 0.0);
            let local_right_ear = ::na::Point3::new(ear_distance/2.0, 0.0, 0.0);

            let position = ::na::Translation3::new(position[0], position[1], z);
            let left_ear = position * local_left_ear;
            let right_ear = position * local_right_ear;

            let mut control = control.lock().unwrap();
            control.effect_volume = effect_volume;
            control.left_ear = left_ear.coords.into();
            control.right_ear = right_ear.coords.into();
        }
    }
}
