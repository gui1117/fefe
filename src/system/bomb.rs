use specs::{Join, ReadStorage, System, WriteStorage};

pub struct BombSystem;

impl<'a> System<'a> for BombSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Contactor>,
        ReadStorage<'a, ::component::Bomb>,
        WriteStorage<'a, ::component::Life>,
    );

    fn run(&mut self, (contactors, bombs, mut lives): Self::SystemData) {
        for (bomb, contactor) in (&bombs, &contactors).join() {
            for &contact in &contactor.0 {
                if let Some(life) = lives.get_mut(contact) {
                    life.0 -= bomb.damage;
                }
            }
        }
    }
}
