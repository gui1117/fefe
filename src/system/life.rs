use specs::{Join, ReadExpect, ReadStorage, System};

pub struct LifeSystem;

impl<'a> System<'a> for LifeSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Life>,
        ReadExpect<'a, ::resource::EntitiesRes>,
    );

    fn run(&mut self, (lives, entities): Self::SystemData) {
        for (life, entity) in (&lives, &*entities).join() {
            if life.0 <= 0 {
                entities.delete(entity).unwrap();
            }
        }
    }
}
