use specs::{ReadExpect, WriteExpect, Join, ReadStorage, System};
use std::f32::EPSILON;

pub struct Boid;

impl<'a> System<'a> for Boid {
    type SystemData = (
        ReadStorage<'a, ::component::Boid>,
        ReadStorage<'a, ::component::RigidBody>,
        ReadExpect<'a, ::resource::EntitiesRes>,
        WriteExpect<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (boids, bodies, entities, mut physic_world): Self::SystemData) {
        for (boid, entity) in (&boids, &*entities).join() {
            let boids_direction = {
                let body = bodies.get(entity);
                if body.is_none() {
                    continue;
                }
                let position = body.unwrap()
                    .get(&physic_world)
                    .position()
                    .translation
                    .vector;
                let mut velocity = ::na::Vector2::new(0.0, 0.0);

                for (other_boid, other_body) in (&boids, &bodies).join() {
                    if other_boid.id == boid.id {
                        let other_body = other_body.get(&physic_world);
                        let other_position = other_body.position().translation.vector;
                        velocity += boid.clamp.compute((other_position - position).norm())
                            * other_body.velocity().linear;
                    }
                }
                velocity.try_normalize(EPSILON)
            };

            if let Some(boids_direction) = boids_direction {
                let mut body = bodies.get(entity).unwrap().get_mut(&mut physic_world);
                let direction =
                    if let Some(direction) = body.velocity().linear.try_normalize(EPSILON) {
                        (direction + boid.weight * boids_direction)
                            .try_normalize(EPSILON)
                            .unwrap_or(::na::zero())
                    } else {
                        boids_direction
                    };

                body.set_linear_velocity(direction * boid.velocity);
            }
        }
    }
}
