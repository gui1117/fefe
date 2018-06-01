use specs::{Join, Fetch, ReadStorage, System, WriteStorage};
use entity::Insertable;

pub struct ChamanSpawnerSystem;

impl<'a> System<'a> for ChamanSpawnerSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Activator>,
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::ChamanSpawner>,
        Fetch<'a, ::resource::PhysicWorld>,
        Fetch<'a, ::resource::LazyUpdate>,
        Fetch<'a, ::resource::EntitiesRes>,
    );

    fn run(&mut self, (activators, bodies, mut chaman_spawner, physic_world, lazy_update, entities): Self::SystemData) {
        for (chaman_spawner, body, activator, entity) in (&mut chaman_spawner, &bodies, &activators, &*entities).join() {
            if activator.activated {
                chaman_spawner.spawned.retain(|spawned| entities.is_alive(*spawned));
                if chaman_spawner.spawned.len() < chaman_spawner.number_of_spawn {
                    let spawn_entity = chaman_spawner.entity.clone();
                    let position = body.get(&physic_world).position().clone();
                    lazy_update.execute(move |world| {
                        let spawned = spawn_entity.insert(position.into(), world);
                        if let Some(chaman_spawner) = world.write::<::component::ChamanSpawner>().get_mut(entity) {
                            chaman_spawner.spawned.push(spawned);
                        }
                    });
                }
            }
        }
    }
}
