use specs::{Join, ReadStorage, System, WriteExpect};
use std::f32::consts::PI;

pub struct VelocityDampingsSystem;

impl<'a> System<'a> for VelocityDampingsSystem {
    type SystemData = (
        ReadStorage<'a, ::component::Player>,
        ReadStorage<'a, ::component::Aim>,
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::VelocityAimDamping>,
        ReadStorage<'a, ::component::VelocityDistanceDamping>,
        WriteExpect<'a, ::resource::PhysicWorld>,
    );

fn run(&mut self, (players, aims, rigid_bodies, aim_dampings, distance_dampings, mut physic_world): Self::SystemData){
        let players_aim = (&players, &aims, &rigid_bodies)
            .join()
            .map(|(_, aim, body)| (aim.0, body.get(&physic_world).position().translation.vector))
            .collect::<Vec<_>>();

        for (aim_damping, rigid_body) in (&aim_dampings, &rigid_bodies).join() {
            let body = rigid_body.get_mut(&mut physic_world);

            let mut velocity = body.velocity().clone();
            // TODO: we probably want the mean when there will be multiple players
            let position = body.position().translation.vector;
            for &(player_aim, ref player_position) in &players_aim {
                let mut v = position - player_position;
                let angle = v[1].atan2(v[0]);
                let mut angle_distance = (angle - player_aim).abs();
                if angle_distance >= PI {
                    angle_distance = 2.0 * PI - angle_distance;
                }
                velocity.linear *= aim_damping.compute(angle_distance);
            }
            body.set_velocity(velocity);
        }
        for (distance_damping, rigid_body) in (&distance_dampings, &rigid_bodies).join() {
            let body = rigid_body.get_mut(&mut physic_world);

            let mut velocity = body.velocity().clone();
            // TODO: we probably want the mean when there will be multiple players
            let position = body.position().translation.vector;
            for &(_, ref player_position) in &players_aim {
                velocity.linear *= distance_damping.compute((player_position - position).norm());
            }
            body.set_velocity(velocity);
        }
    }
}
