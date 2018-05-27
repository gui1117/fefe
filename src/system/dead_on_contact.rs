use specs::{Join, ReadStorage, System, WriteStorage};

pub struct DeadOnContactSystem;

impl<'a> System<'a> for DeadOnContactSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Contactor>,
        ReadStorage<'a, ::component::DeadOnContact>,
        WriteStorage<'a, ::component::Life>,
    );

    fn run(&mut self, (contactors, dead_on_contacts, mut lives): Self::SystemData) {
        for (_, contactor, life) in (&dead_on_contacts, &contactors, &mut lives).join() {
            if !contactor.0.is_empty() {
                life.0 = 0;
            }
        }
    }
}
