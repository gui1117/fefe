use specs::{Join, Fetch, ReadStorage, System, WriteStorage};
use entity::Insertable;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Normal};

pub struct ChamanSpawnerSystem;

impl<'a> System<'a> for ChamanSpawnerSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::ChamanSpawner>,
        Fetch<'a, ::resource::PhysicWorld>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::resource::LazyUpdate>,
        Fetch<'a, ::resource::EntitiesRes>,
    );

    fn run(&mut self, (bodies, mut chaman_spawner, physic_world, update_time, lazy_update, entities): Self::SystemData) {
        for (chaman_spawner, body, entity) in (&mut chaman_spawner, &bodies, &*entities).join() {
            chaman_spawner.next_spawn = if let Some(mut next_spawn) = chaman_spawner.next_spawn {
                next_spawn -= update_time.0;
                if next_spawn <= 0.0 {
                    let spawn_entity = chaman_spawner.entity.clone();
                    let position = body.get(&physic_world).position().clone();
                    lazy_update.execute(move |world| {
                        let spawned = spawn_entity.insert(position.into(), world);
                        if let Some(chaman_spawner) = world.write::<::component::ChamanSpawner>().get_mut(entity) {
                            chaman_spawner.spawned.push(spawned);
                        }
                    });
                    None
                } else {
                    Some(next_spawn)
                }
            } else {
                chaman_spawner.spawned.retain(|spawned| entities.is_alive(*spawned));
                if chaman_spawner.spawned.len() < chaman_spawner.number_of_spawn {
                    Some(Normal::new(chaman_spawner.spawn_time.0, chaman_spawner.spawn_time.1).ind_sample(&mut thread_rng()) as f32)
                } else {
                    None
                }
            };
        }
    }
}
