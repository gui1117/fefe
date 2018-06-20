use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use rodio::source::Source;

use rodio::Sample;

/// Builds a new mixer.
///
/// Added sources must follow specified channels and sample rate
///
/// After creating a mixer, you can add new sounds with the controller.
pub fn mixer<S>(
    channels: u16, sample_rate: u32,
) -> (Arc<DynamicMixerController<S>>, DynamicMixer<S>)
where
    S: Sample + Send + 'static,
{
    let input = Arc::new(DynamicMixerController {
        has_pending: AtomicBool::new(false),
        pending_sources: Mutex::new(Vec::new()),
        channels: channels,
        sample_rate: sample_rate,
    });

    let output = DynamicMixer {
        current_sources: Vec::with_capacity(16),
        input: input.clone(),
        to_drop: vec![],
    };

    (input, output)
}

/// The input of the mixer.
pub struct DynamicMixerController<S> {
    has_pending: AtomicBool,
    pending_sources: Mutex<Vec<Box<Source<Item = S> + Send>>>,
    channels: u16,
    sample_rate: u32,
}

impl<S> DynamicMixerController<S>
where
    S: Sample + Send + 'static,
{
    /// Adds a new source to mix to the existing ones.
    #[inline]
    pub fn add<T>(&self, source: T)
    where
        T: Source<Item = S> + Send + 'static,
    {
        assert_eq!(source.channels(), self.channels);
        assert_eq!(source.sample_rate(), self.sample_rate);

        self.pending_sources
            .lock()
            .unwrap()
            .push(Box::new(source));
        self.has_pending.store(true, Ordering::SeqCst); // TODO: can we relax this ordering?
    }
}

/// The output of the mixer. Implements `Source`.
pub struct DynamicMixer<S> {
    // The current iterator that produces samples.
    current_sources: Vec<Box<Source<Item = S> + Send>>,
    to_drop: Vec<usize>,

    // The pending sounds.
    input: Arc<DynamicMixerController<S>>,
}

impl<S> Source for DynamicMixer<S>
where
    S: Sample + Send + 'static,
{
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        self.input.channels
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.input.sample_rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

impl<S> Iterator for DynamicMixer<S>
where
    S: Sample + Send + 'static,
{
    type Item = S;

    #[inline]
    fn next(&mut self) -> Option<S> {
        if self.input.has_pending.load(Ordering::SeqCst) {
            // TODO: relax ordering?
            let mut pending = self.input.pending_sources.lock().unwrap();
            self.current_sources.extend(pending.drain(..));
            self.input.has_pending.store(false, Ordering::SeqCst); // TODO: relax ordering?
        }

        let mut sum = S::zero_value();
        for (num, src) in self.current_sources.iter_mut().enumerate() {
            if let Some(val) = src.next() {
                sum = sum.saturating_add(val);
            } else {
                self.to_drop.push(num);
            }
        }

        for td in self.to_drop.drain(..).rev() {
            self.current_sources.remove(td);
        }

        Some(sum)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}
