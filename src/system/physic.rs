use specs::{Fetch, FetchMut, System};

pub struct PhysicSystem;

impl<'a> System<'a> for PhysicSystem {
    type SystemData = (
        Fetch<'a, ::resource::UpdateTime>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (update_time, mut physic_world): Self::SystemData) {
        let mut remaining_to_update = update_time.0;
        while remaining_to_update > ::CFG.physic_min_timestep + ::std::f32::EPSILON {
            let timestep = remaining_to_update.min(::CFG.physic_max_timestep);
            remaining_to_update -= timestep;
            physic_world.set_timestep(timestep);
            physic_world.step();
            for i in physic_world.contact_events() {
                println!("{:?}", i);
            }
        }
    }
}
