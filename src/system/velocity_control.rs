use nphysics2d::math::Velocity;
use specs::{WriteExpect, Join, ReadStorage, System};

pub struct VelocityControlSystem;

impl<'a> System<'a> for VelocityControlSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::VelocityControl>,
        WriteExpect<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (rigid_bodies, velocity_controls, mut physic_world): Self::SystemData) {
        for (velocity_control, rigid_body) in (&velocity_controls, &rigid_bodies).join() {
            rigid_body
                .get_mut(&mut physic_world)
                .set_velocity(Velocity {
                    linear: velocity_control.direction * velocity_control.velocity,
                    angular: 0.0,
                })
        }
    }
}
