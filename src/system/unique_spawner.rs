use specs::{Join, Fetch, ReadStorage, System, WriteStorage};
use entity::Insertable;
use ncollide2d::world::CollisionGroups;
use ncollide2d::query::Ray;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};

pub struct UniqueSpawnerSystem;

impl<'a> System<'a> for UniqueSpawnerSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::UniqueSpawner>,
        Fetch<'a, ::resource::PhysicWorld>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::resource::LazyUpdate>,
        Fetch<'a, ::resource::EntitiesRes>,
        Fetch<'a, ::resource::BodiesMap>,
    );

    fn run(&mut self, (players, bodies, mut unique_spawners, physic_world, update_time, lazy_update, entities, bodies_map): Self::SystemData) {
        let mut rng = thread_rng();
        let range_0_1 = Range::new(0.0, 1.0);
        let players_position = (&players, &bodies)
            .join()
            .map(|(_, body)| body.get(&physic_world).position().translation.vector)
            .collect::<Vec<_>>();

        for (unique_spawner, body, entity) in (&mut unique_spawners, &bodies, &*entities).join() {
            unique_spawner.next_refreash -= update_time.0;
            if unique_spawner.next_refreash <= 0.0 {
                unique_spawner.next_refreash = ::component::UNIQUE_SPAWNER_TIMER;
                let position = body.get(&physic_world).position().clone();
                let pos_vector = position.translation.vector;
                for player_position in &players_position {
                    let dist_vector = player_position - pos_vector;
                    if range_0_1.ind_sample(&mut rng) <= unique_spawner.proba.compute(dist_vector.norm()) {
                        let ray = Ray::new(::na::Point::from_coordinates(pos_vector), dist_vector);
                        // TODO: collision groups
                        let collision_groups = CollisionGroups::new();
                        let mut interferences = physic_world.collision_world().interferences_with_ray(&ray, &collision_groups);
                        if let Some((object, _)) = interferences.next() {
                            if players.get(*bodies_map.get(&object.data().body()).unwrap()).is_some() {
                                entities.delete(entity).unwrap();
                                let spawn_entity = unique_spawner.entity.clone();
                                lazy_update.execute(move |world| {
                                    spawn_entity.insert(position.into(), world);
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}
