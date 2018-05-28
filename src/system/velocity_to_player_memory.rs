use specs::{Join, Fetch, FetchMut, ReadStorage, System, WriteStorage};
use ncollide2d::world::CollisionGroups;
use ncollide2d::query::Ray;

pub struct VelocityToPlayerMemorySystem;

impl<'a> System<'a> for VelocityToPlayerMemorySystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::VelocityToPlayerMemory>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::resource::BodiesMap>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (players, rigid_bodies, mut vtpms, update_time, bodies_map, mut physic_world): Self::SystemData) {
        let players_position = (&players, &rigid_bodies)
            .join()
            .map(|(_, body)| body.get(&physic_world).position().translation.vector)
            .collect::<Vec<_>>();

        for (vtpm, rigid_body) in (&mut vtpms, &rigid_bodies).join() {
            vtpm.next_refreash -= update_time.0;
            let position = rigid_body.get(&physic_world).position().translation.vector;
            if vtpm.next_refreash <= 0.0 {
                vtpm.next_refreash = ::component::VELOCITY_TO_PLAYER_MEMORY_REFREASH_RATE;
                let closest_in_sight = players_position.iter()
                    .filter_map(|player_position| {
                        let ray = Ray::new(::na::Point::from_coordinates(position), player_position - position);
                        // TODO: collision groups
                        let collision_groups = CollisionGroups::new();
                        let mut interferences = physic_world.collision_world().interferences_with_ray(&ray, &collision_groups);
                        if let Some((object, _)) = interferences.next() {
                            if players.get(*bodies_map.get(&object.data().body()).unwrap()).is_some() {
                                Some(object.position().translation.vector)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .min_by_key(|vector| vector.norm() as usize);

                vtpm.last_closest_in_sight = closest_in_sight.or(vtpm.last_closest_in_sight);
            }

            if let Some(last_closest_in_sight) = vtpm.last_closest_in_sight {
                let d = ::component::VELOCITY_TO_PLAYER_DISTANCE_TO_GOAL;
                let v = (last_closest_in_sight - position).try_normalize(d).unwrap_or(::na::zero()) * vtpm.velocity;
                rigid_body.get_mut(&mut physic_world).set_linear_velocity(v);
            }
        }
    }
}
