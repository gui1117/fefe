use specs::{Join, ReadStorage, System, WriteStorage};

pub struct ContactDamageSystem;

impl<'a> System<'a> for ContactDamageSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Contactor>,
        ReadStorage<'a, ::component::ContactDamage>,
        WriteStorage<'a, ::component::Life>,
    );

    fn run(&mut self, (contactors, damages, mut lives): Self::SystemData) {
        for (damage, contactor) in (&damages, &contactors).join() {
            if !contactor.0.is_empty() {
                for &contact in &contactor.0 {
                    if let Some(life) = lives.get_mut(contact) {
                        life.0 -= damage.0;
                    }
                }
            }
        }
    }
}
