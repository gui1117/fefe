use specs::{Join, ReadExpect, System, ReadStorage, WriteExpect, WriteStorage};
use nphysics2d::algebra::Velocity2;

pub struct PositionInPathSystem;

impl<'a> System<'a> for PositionInPathSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::PositionInPath>,
        ReadExpect<'a, ::resource::UpdateTime>,
        WriteExpect<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (bodies, mut paths, update_time, mut physic_world): Self::SystemData) {
        for (path, body) in (&mut paths, &bodies).join() {
            path.current_advancement += update_time.0*path.velocity;
            while path.current_advancement >= path.distances[path.current_point] {
                path.current_advancement -= path.distances[path.current_point];
                path.current_point = (path.current_point + 1) % path.points.len();
            }

            let body = body.get_mut(&mut physic_world);
            let line = (path.points[(path.current_point + 1)%path.points.len()] - path.points[path.current_point]).normalize();
            let displacment = path.points[path.current_point] + path.current_advancement*line - body.position().translation.vector;
            body.apply_displacement(&Velocity2::new(displacment, 0.0));
            body.set_velocity(Velocity2::zero());
        }
    }
}
