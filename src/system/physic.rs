use specs::{WriteStorage, Fetch, FetchMut, System, Join};

pub struct PhysicSystem;

impl<'a> System<'a> for PhysicSystem {
    type SystemData = (
        WriteStorage<'a, ::component::RigidBody>,
        Fetch<'a, ::resource::UpdateTime>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (mut rigid_bodies, update_time, mut physic_world): Self::SystemData) {
        // for body in (&mut rigid_bodies).join() {
        //     let body = body.get_mut(&mut physic_world);
        //     let velocity  = *body.velocity();
        //     body.set_velocity(velocity*1.0);
        // }
        let mut remaining_to_update = update_time.0;
        while remaining_to_update > ::CFG.physic_min_timestep + ::std::f32::EPSILON {
            let timestep = remaining_to_update.min(::CFG.physic_max_timestep);
            remaining_to_update -= timestep;
            physic_world.set_timestep(timestep);
            physic_world.step();
            for i in physic_world.contact_events() {
                // TODO: collider ?
            }
        }
        // for body in (&mut rigid_bodies).join() {
        //     body.get_mut(&mut physic_world).clear_dynamics();
        // }
    }
}
