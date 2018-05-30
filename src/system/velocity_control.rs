use specs::{Join, FetchMut, ReadStorage, System};
use nphysics2d::math::Velocity;

pub struct VelocityControl;

impl<'a> System<'a> for VelocityControl {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::VelocityControl>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (rigid_bodies, velocity_controls, mut physic_world): Self::SystemData) {
        for (velocity_control, rigid_body) in (&velocity_controls, &rigid_bodies).join() {
            rigid_body.get_mut(&mut physic_world).set_velocity(Velocity {
                linear: velocity_control.direction * velocity_control.velocity,
                angular: 0.0,
            })
        }
    }
}
