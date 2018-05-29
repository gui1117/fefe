use specs::{Join, Fetch, FetchMut, ReadStorage, System};

pub struct Camera;

impl<'a> System<'a> for Camera {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::Player>,
        Fetch<'a, ::resource::PhysicWorld>,
        FetchMut<'a, ::resource::Camera>,
    );

    fn run(&mut self, (bodies, players, physic_world, mut camera): Self::SystemData) {
        if let Some((_, body)) = (&players, &bodies).join().next() {
            camera.position.translation = body.get(&physic_world).position().translation;
        }
    }
}
