use specs::{Fetch, FetchMut, Join, System, WriteStorage};

pub struct ActivatorSystem;

impl<'a> System<'a> for ActivatorSystem {
    type SystemData = (
        WriteStorage<'a, ::component::Activators>,
        Fetch<'a, ::resource::UpdateTime>,
        FetchMut<'a, ::resource::Tempos>,
    );

    fn run(&mut self, (mut activatorses, update_time, mut tempos): Self::SystemData) {
        for activator in (&mut activatorses).join().flat_map(|a| a.0.iter_mut()) {
            activator.activated = false;
        }

        for (id, tempo) in tempos.iter_mut().enumerate() {
            tempo.next_beat_time -= update_time.0;
            while tempo.next_beat_time <= 0.0 {
                tempo.next_beat_time += tempo.time;
                for activator in (&mut activatorses).join().flat_map(|a| a.0.iter_mut()) {
                    if activator.tempo == id {
                        activator.activated =
                            activator.partition[tempo.beat % activator.partition.len()];
                    }
                }
                tempo.beat += 1;
            }
        }
    }
}
