use specs::{Fetch, FetchMut, Join, System, WriteStorage};

pub struct ActivatorSystem;

impl<'a> System<'a> for ActivatorSystem {
    type SystemData = (
        WriteStorage<'a, ::component::Activator>,
        Fetch<'a, ::resource::UpdateTime>,
        FetchMut<'a, ::resource::Tempos>,
    );

    fn run(&mut self, (mut activators, update_time, mut tempos): Self::SystemData) {
        for activator in (&mut activators).join() {
            activator.activated = false;
        }

        for (id, tempo) in tempos.iter_mut().enumerate() {
            tempo.next_beat_time -= update_time.0;
            while tempo.next_beat_time <= 0.0 {
                tempo.next_beat_time += tempo.time;
                for activator in (&mut activators).join() {
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
