use specs::{Join, ReadStorage, System, WriteStorage};

pub struct ContactDamageSystem;

impl<'a> System<'a> for ContactDamageSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::Contactor>,
        ReadStorage<'a, ::component::ContactDamage>,
        WriteStorage<'a, ::component::Life>,
    );

    fn run(&mut self, (players, contactors, damages, mut lives): Self::SystemData) {
        for (damage, contactor) in (&damages, &contactors).join() {
            if !contactor.0.is_empty() {
                for &contact in &contactor.0 {
                    match (players.get(contact), lives.get_mut(contact)) {
                        (Some(_), Some(life)) => life.0 -= damage.0,
                        _ => (),
                    }
                }
            }
        }
    }
}
