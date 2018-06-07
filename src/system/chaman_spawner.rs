use entity::Insertable;
use specs::{Join, ReadExpect, ReadStorage, System, WriteStorage};

pub struct ChamanSpawnerSystem;

impl<'a> System<'a> for ChamanSpawnerSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Activators>,
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::ChamanSpawner>,
        ReadExpect<'a, ::resource::PhysicWorld>,
        ReadExpect<'a, ::resource::LazyUpdate>,
        ReadExpect<'a, ::resource::EntitiesRes>,
        ReadExpect<'a, ::resource::InsertablesMap>,
    );

    fn run(
        &mut self,
        (
            activatorses,
            bodies,
            mut chaman_spawner,
            physic_world,
            lazy_update,
            entities,
            insertables_map,
        ): Self::SystemData,
    ) {
        for (chaman_spawner, body, activators, entity) in
            (&mut chaman_spawner, &bodies, &activatorses, &*entities).join()
        {
            if activators[chaman_spawner.activator].activated {
                chaman_spawner
                    .spawned
                    .retain(|spawned| entities.is_alive(*spawned));
                if chaman_spawner.spawned.len() < chaman_spawner.number_of_spawn {
                    let spawn = insertables_map.get(&chaman_spawner.spawn).unwrap().clone();
                    let position = body.get(&physic_world).position().clone();
                    lazy_update.exec(move |world| {
                        let spawned = spawn.insert(position.into(), world);
                        if let Some(chaman_spawner) = world
                            .write_storage::<::component::ChamanSpawner>()
                            .get_mut(entity)
                        {
                            chaman_spawner.spawned.push(spawned);
                        }
                    });
                }
            }
        }
    }
}
