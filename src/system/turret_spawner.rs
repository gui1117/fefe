use entity::Insertable;
use specs::{Join, ReadExpect, ReadStorage, WriteStorage, System};
use std::f32::consts::PI;

pub struct TurretSpawnerSystem;

impl<'a> System<'a> for TurretSpawnerSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::Activators>,
        WriteStorage<'a, ::component::TurretSpawner>,
        ReadExpect<'a, ::resource::PhysicWorld>,
        ReadExpect<'a, ::resource::Tempos>,
        ReadExpect<'a, ::resource::LazyUpdate>,
        ReadExpect<'a, ::resource::InsertablesMap>,
        ReadExpect<'a, ::resource::Audio>,
    );

    fn run(
        &mut self,
        (bodies, activatorses, mut turret_spawners, physic_world, tempos, lazy_update, insertables_map, audio): Self::SystemData,
){
        for (turret_spawner, activators, body) in (&mut turret_spawners, &activatorses, &bodies).join()
        {
            for turret_part in turret_spawner.iter_mut() {
                let ref activator = activators[turret_part.activator];
                let ref tempo = tempos[activator.tempo];

                if let Some(id) = turret_part.rotation_activator {
                    let ref rotation_activator = activators[id];
                    if rotation_activator.activated {
                        let mut position = body.get(&physic_world).position();
                        audio.play(rotation_activator.sound, position.translation.vector.into());
                        turret_part.clockwise = !turret_part.clockwise;
                        // TODO: modulo ...
                        turret_part.start_time = -(turret_part.start_time+tempo.beat as isize * 2);
                    }
                }

                if activator.activated {
                    let mut position = body.get(&physic_world).position();
                    audio.play(activator.sound, position.translation.vector.into());

                    let mut angle = (2.0 * PI / turret_part.rotation_time as f32)
                        * (turret_part.start_time + tempo.beat as isize) as f32;

                    if turret_part.clockwise {
                        angle *= -1.0;
                    }

                    position.rotation *= ::na::UnitComplex::new(angle);
                    ::util::move_forward(&mut position, turret_part.shoot_distance);
                    let spawn = insertables_map.get(&turret_part.spawn).unwrap().clone();
                    lazy_update.exec(move |world| {
                        spawn.insert(position.into(), world);
                    });
                }
            }
        }
    }
}
