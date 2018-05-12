use specs::{Join, Fetch, ReadStorage, System, WriteStorage};

pub struct BombSystem;

impl<'a> System<'a> for BombSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Contactor>,
        ReadStorage<'a, ::component::Bomb>,
        WriteStorage<'a, ::component::Life>,
        Fetch<'a, ::specs::EntitiesRes>,
    );

    fn run(&mut self, (contactors, bombs, mut lives, entities): Self::SystemData) {
        for (bomb, contactor, entity) in (&bombs, &contactors, &*entities).join() {
            if !contactor.0.is_empty() {
                *lives.get_mut(entity).unwrap() = 0.into();
                for &contact in &contactor.0 {
                    if let Some(life) = lives.get_mut(contact) {
                        life.0 -= bomb.damage;
                    }
                }
            }
        }
    }
}
