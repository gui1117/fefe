use ncollide2d::query::Ray;
use ncollide2d::world::CollisionGroups;
use nphysics2d::math::Velocity;
use rand::distributions::{IndependentSample, Range};
use rand::thread_rng;
use specs::{Fetch, FetchMut, Join, ReadStorage, System, WriteStorage};
use std::f32::consts::PI;
use std::f32::EPSILON;

pub struct VelocityToPlayerRandomSystem;

impl<'a> System<'a> for VelocityToPlayerRandomSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::Aim>,
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::Activators>,
        WriteStorage<'a, ::component::VelocityToPlayerRandom>,
        Fetch<'a, ::resource::BodiesMap>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

fn run(&mut self, (players, aims, rigid_bodies, activatorses, mut vtprs, bodies_map, mut physic_world): Self::SystemData){
        let mut rng = thread_rng();
        let range_0_1 = Range::new(0.0, 1.0);
        let players_position = (&players, &rigid_bodies)
            .join()
            .map(|(_, body)| body.get(&physic_world).position().translation.vector)
            .collect::<Vec<_>>();

        for (vtpr, rigid_body, activators) in (&mut vtprs, &rigid_bodies, &activatorses).join() {
            let position = rigid_body.get(&physic_world).position().translation.vector;
            if activators[vtpr.activator].activated {
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
                        if let Some((object, _)) = interference {
                            let collided_entity = *bodies_map.get(&object.data().body()).unwrap();
                            match (players.get(collided_entity), aims.get(collided_entity)) {
                                (Some(_), Some(aim)) => {
                                    Some((aim, object.position().translation.vector - position))
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    })
                    .min_by_key(|(_, distance)| (distance.norm() * ::CMP_PRECISION) as usize);

                vtpr.current_direction = closest_in_sight
                    .and_then(|(aim, distance)| {
                        let angle = distance[1].atan2(distance[0]);
                        let mut angle_distance = (angle - aim.0).abs() % 2.0 * PI;
                        if angle_distance >= PI {
                            angle_distance = 2.0 * PI - angle_distance;
                        }
                        let final_proba = vtpr.dist_proba_clamp.compute(distance.norm())
                            * vtpr.aim_proba_clamp.compute(angle_distance);

                        if range_0_1.ind_sample(&mut rng) <= final_proba {
                            let mut direction =
                                distance.try_normalize(EPSILON).unwrap_or(::na::zero());

                            if !vtpr.toward_player {
                                direction *= -1.0;
                            }

                            if let Some(weight) = vtpr.random_weighted {
                                direction += ::util::random_normalized(&mut rng) * weight;
                                direction.normalize();
                            }
                            Some(direction)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| ::util::random_normalized(&mut rng))
                    .try_normalize(EPSILON)
                    .unwrap_or(::na::zero());
            }

            rigid_body
                .get_mut(&mut physic_world)
                .set_velocity(Velocity {
                    linear: vtpr.current_direction * vtpr.velocity,
                    angular: 0.0,
                });
        }
    }
}
