use specs::{ReadStorage, Fetch, FetchMut, System, Join};
use nphysics2d::math::Force;
use nphysics2d::object::BodyHandle;

pub struct PhysicSystem {
    step_forces_cache: Vec<(BodyHandle, Force<f32>)>,
}

impl PhysicSystem {
    pub fn new() -> Self {
        PhysicSystem {
            step_forces_cache: vec![],
        }
    }
}

impl<'a> System<'a> for PhysicSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::ControlForce>,
        ReadStorage<'a, ::component::Damping>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::specs::EntitiesRes>,
        Fetch<'a, ::resource::StepForces>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(&mut self, (rigid_bodies, control_forces, dampings, update_time, entities, step_forces, mut physic_world): Self::SystemData) {
        {
            self.step_forces_cache.clear();
            for (body, entity) in (&rigid_bodies, &*entities).join() {
                let body = body.get(&physic_world);

                let mut force = Force::zero();
                let mut linear_damping = 0.0;
                let mut angular_damping = 0.0;

                if let Some(control_force) = control_forces.get(entity) {
                    force += control_force.0;
                }
                if let Some(damping) = dampings.get(entity) {
                    linear_damping += damping.linear;
                    angular_damping += damping.angular;
                }

                let velocity  = body.velocity();
                force += Force {
                    linear: -velocity.linear*linear_damping,
                    angular: -velocity.angular*angular_damping,
                };

                if force.linear != ::na::zero() || force.angular != 0.0 {
                    self.step_forces_cache.push((body.handle(), force));
                }
            }
            let mut step_forces = step_forces.get_mut(&mut physic_world);
            ::std::mem::swap(&mut step_forces.forces, &mut self.step_forces_cache);
        }

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
    }
}
