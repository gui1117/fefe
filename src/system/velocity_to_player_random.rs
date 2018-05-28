use specs::{Join, Fetch, FetchMut, ReadStorage, System, WriteStorage};
use ncollide2d::world::CollisionGroups;
use ncollide2d::query::Ray;
use rand::{thread_rng, Rand};
use rand::distributions::{IndependentSample, Range, Normal};
use std::f32::EPSILON;

pub struct VelocityToPlayerRandomSystem;

impl<'a> System<'a> for VelocityToPlayerRandomSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::VelocityToPlayerRandom>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::resource::BodiesMap>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (players, rigid_bodies, mut vtprs, update_time, bodies_map, mut physic_world): Self::SystemData) {
        let mut rng = thread_rng();
        let range_0_1 = Range::new(0.0, 1.0);
        let players_position = (&players, &rigid_bodies)
            .join()
            .map(|(_, body)| body.get(&physic_world).position().translation.vector)
            .collect::<Vec<_>>();

        for (vtpr, rigid_body) in (&mut vtprs, &rigid_bodies).join() {
            vtpr.next_refreash -= update_time.0;
            let position = rigid_body.get(&physic_world).position().translation.vector;
            if vtpr.next_refreash <= 0.0 {
                let distribution = Normal::new(vtpr.refreash_time.0, vtpr.refreash_time.1);
                vtpr.next_refreash = distribution.ind_sample(&mut rng) as f32;
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

                let direction = closest_in_sight.and_then(|closest_in_sight| {
                    let direction = closest_in_sight - position;

                    if range_0_1.ind_sample(&mut rng) <= vtpr.proba.compute(direction.norm()) {
                        let mut final_direction = direction
                            .try_normalize(EPSILON)
                            .unwrap_or(::na::zero());

                        if vtpr.toward_player {
                            final_direction *= -1.0;
                        }

                        if let Some(weight) = vtpr.random_weighted {
                            final_direction += ::na::Vector2::rand(&mut rng).normalize()*weight;
                            final_direction.normalize();
                        }
                        Some(final_direction)
                    } else {
                        None
                    }
                }).unwrap_or_else(|| ::na::Vector2::rand(&mut rng));

                rigid_body.get_mut(&mut physic_world).set_linear_velocity(direction.normalize() * vtpr.velocity);
            }
        }
    }
}
