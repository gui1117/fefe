use entity::Insertable;
use specs::{Fetch, Join, ReadStorage, System};
use std::f32::consts::PI;

pub struct TurretPartSpawnerSystem;

impl<'a> System<'a> for TurretPartSpawnerSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::TurretPartSpawner>,
        ReadStorage<'a, ::component::Activator>,
        Fetch<'a, ::resource::PhysicWorld>,
        Fetch<'a, ::resource::Tempos>,
        Fetch<'a, ::resource::LazyUpdate>,
        Fetch<'a, ::resource::InsertablesMap>,
    );

    fn run(
        &mut self,
        (bodies, turret_parts, activators, physic_world, tempos, lazy_update, insertables_map): Self::SystemData,
    ) {
        for (turret_part, activator) in (&turret_parts, &activators).join() {
            if activator.activated {
                let mut position = bodies.get(turret_part.body).unwrap().get(&physic_world).position();
                let ref tempo = tempos[activator.tempo];
                let mut angle = (2.0 * PI / turret_part.rotation_time as f32) * (turret_part.start_time + tempo.beat) as f32;
                if turret_part.clockwise {
                    angle *= -1.0;
                }

                position.rotation *= ::na::UnitComplex::new(angle);
                ::util::move_forward(&mut position, turret_part.shoot_distance);
                let spawn = insertables_map.get(&turret_part.spawn).unwrap().clone();
                lazy_update.execute(move |world| {
                    spawn.insert(position.into(), world);
                });
            }
        }
    }
}
