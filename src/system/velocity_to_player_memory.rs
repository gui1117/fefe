use ncollide2d::query::Ray;
use ncollide2d::world::CollisionGroups;
use nphysics2d::math::Velocity;
use specs::{ReadExpect, WriteExpect, Join, ReadStorage, System, WriteStorage};

pub struct VelocityToPlayerMemorySystem;

// IDEA: maybe use the last seen rotation or just current velocity
//       and continue on the path a little
impl<'a> System<'a> for VelocityToPlayerMemorySystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::Activators>,
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::VelocityToPlayerMemory>,
        ReadExpect<'a, ::resource::BodiesMap>,
        WriteExpect<'a, ::resource::PhysicWorld>,
    );

fn run(&mut self, (players, activatorses, rigid_bodies, mut vtpms, bodies_map, mut physic_world): Self::SystemData){
        let players_position = (&players, &rigid_bodies)
            .join()
            .map(|(_, body)| body.get(&physic_world).position().translation.vector)
            .collect::<Vec<_>>();

        for (vtpm, rigid_body, activators) in (&mut vtpms, &rigid_bodies, &activatorses).join() {
            let position = rigid_body.get(&physic_world).position().translation.vector;

            if activators[vtpm.activator].activated {
                let closest_in_sight = players_position
                    .iter()
                    .filter_map(|player_position| {
                        let ray = Ray::new(
                            ::na::Point::from_coordinates(position),
                            player_position - position,
                        );
                        let mut collision_groups = CollisionGroups::new();
                        collision_groups.set_whitelist(&[
                            ::entity::Group::Wall as usize,
                            ::entity::Group::Player as usize,
                        ]);
                        let interference = physic_world
                            .collision_world()
                            .interferences_with_ray(&ray, &collision_groups)
                            .min_by_key(|(_, intersection)| {
                                (intersection.toi * ::CMP_PRECISION) as isize
                            });
                        if let Some((object, intersection)) = interference {
                            if players
                                .get(*bodies_map.get(&object.data().body()).unwrap())
                                .is_some()
                            {
                                Some((object.position().translation.vector, intersection.toi))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .min_by_key(|(_, toi)| (toi * ::CMP_PRECISION) as isize)
                    .map(|(object_position, _)| object_position);

                if vtpm.memory {
                    vtpm.last_closest_in_sight = closest_in_sight.or(vtpm.last_closest_in_sight);
                } else {
                    vtpm.last_closest_in_sight = closest_in_sight;
                }
            }

            let direction = if let Some(last_closest_in_sight) = vtpm.last_closest_in_sight.clone()
            {
                let d = ::component::VELOCITY_TO_PLAYER_DISTANCE_TO_GOAL;
                if let Some(direction) = (last_closest_in_sight - position).try_normalize(d) {
                    direction
                } else {
                    vtpm.last_closest_in_sight = None;
                    ::na::zero()
                }
            } else {
                ::na::zero()
            };

            rigid_body
                .get_mut(&mut physic_world)
                .set_velocity(Velocity {
                    linear: direction * vtpm.velocity,
                    angular: 0.0,
                });
        }
    }
}
