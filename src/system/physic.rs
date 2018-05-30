use ncollide2d::events::ContactEvent;
use nphysics2d::math::Force;
use nphysics2d::object::BodyHandle;
use specs::{Fetch, FetchMut, Join, ReadStorage, System, WriteStorage};
use std::f32::consts::PI;

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
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::Aim>,
        ReadStorage<'a, ::component::ControlForce>,
        ReadStorage<'a, ::component::Damping>,
        ReadStorage<'a, ::component::GravityToPlayers>,
        ReadStorage<'a, ::component::PlayersAimDamping>,
        ReadStorage<'a, ::component::PlayersDistanceDamping>,
        WriteStorage<'a, ::component::Contactor>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::specs::EntitiesRes>,
        Fetch<'a, ::resource::StepForces>,
        Fetch<'a, ::resource::BodiesMap>,
        Fetch<'a, ::resource::Conf>,
        FetchMut<'a, ::resource::PhysicWorld>,
    );

    fn run(
        &mut self,
        (
            players,
            rigid_bodies,
            aims,
            control_forces,
            dampings,
            gravities_to_players,
            players_aim_dampings,
            players_distance_dampings,
            mut contactors,
            update_time,
            entities,
            step_forces,
            bodies_map,
            conf,
            mut physic_world,
        ): Self::SystemData,
    ) {
        {
            let players_aim = (&players, &aims, &rigid_bodies)
                .join()
                .map(|(_, aim, body)| {
                    (aim.0, body.get(&physic_world).position().translation.vector)
                })
                .collect::<Vec<_>>();

            let players_position = (&players, &rigid_bodies)
                .join()
                .map(|(_, body)| body.get(&physic_world).position())
                .collect::<Vec<_>>();

            self.step_forces_cache.clear();
            for (body, entity) in (&rigid_bodies, &*entities).join() {
                let body = body.get(&physic_world);

                let mut force = Force::zero();
                if let Some(control_force) = control_forces.get(entity) {
                    force += control_force.0;
                }
                if let Some(gravity_to_players) = gravities_to_players.get(entity) {
                    let position = body.position().translation.vector;
                    let powi = gravity_to_players.powi;

                    for player_position in &players_position {
                        let mut v = player_position.translation.vector - position;
                        v *= gravity_to_players.force / v.norm().powi(powi + 1);
                        force += Force::linear(v);
                    }
                }

                let mut linear_damping = 0.0;
                let mut angular_damping = 0.0;
                if let Some(damping) = dampings.get(entity) {
                    linear_damping += damping.linear;
                    angular_damping += damping.angular;
                }
                if let Some(player_aim_damping) = players_aim_dampings.get(entity) {
                    // TODO: we probably want the mean when there will be multiple players
                    let position = body.position().translation.vector;
                    for &(player_aim, ref player_position) in &players_aim {
                        let mut v = player_position - position;
                        let angle = v[1].atan2(v[0]);
                        let mut angle_distance = (angle - player_aim).abs() % 2.0 * PI;
                        if angle_distance >= PI {
                            angle_distance = 2.0 * PI - angle_distance;
                        }
                        linear_damping /= player_aim_damping.compute(angle_distance);
                    }
                }
                if let Some(player_distance_damping) = players_distance_dampings.get(entity) {
                    // TODO: we probably want the mean when there will be multiple players
                    let position = body.position().translation.vector;
                    for &(_, ref player_position) in &players_aim {
                        linear_damping /= player_distance_damping.compute((player_position - position).norm());
                    }
                }

                let velocity = body.velocity();
                force += Force {
                    linear: -velocity.linear * linear_damping,
                    angular: -velocity.angular * angular_damping,
                };

                if force.linear != ::na::zero() || force.angular != 0.0 {
                    self.step_forces_cache.push((body.handle(), force));
                }
            }
            let step_forces = step_forces.get_mut(&mut physic_world);
            ::std::mem::swap(&mut step_forces.forces, &mut self.step_forces_cache);
        }

        let mut remaining_to_update = update_time.0;
        while remaining_to_update > conf.physic_min_timestep + ::std::f32::EPSILON {
            let timestep = remaining_to_update.min(conf.physic_max_timestep);
            remaining_to_update -= timestep;
            physic_world.set_timestep(timestep);
            physic_world.step();
            for contact in physic_world.contact_events() {
                let collision_world = physic_world.collision_world();
                match contact {
                    &ContactEvent::Started(coh1, coh2) => {
                        let bh1 = collision_world
                            .collision_object(coh1)
                            .unwrap()
                            .data()
                            .body();
                        let bh2 = collision_world
                            .collision_object(coh2)
                            .unwrap()
                            .data()
                            .body();
                        let e1 = *bodies_map.get(&bh1).unwrap();
                        let e2 = *bodies_map.get(&bh2).unwrap();
                        if let Some(contactor) = contactors.get_mut(e1) {
                            contactor.push(e2);
                        }
                        if let Some(contactor) = contactors.get_mut(e2) {
                            contactor.push(e1);
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
