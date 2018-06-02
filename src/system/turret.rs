use entity::Insertable;
use specs::{Fetch, Join, ReadStorage, System, WriteStorage};

pub struct TurretSystem;

impl<'a> System<'a> for TurretSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::Turret>,
        Fetch<'a, ::resource::PhysicWorld>,
        Fetch<'a, ::resource::UpdateTime>,
        Fetch<'a, ::specs::LazyUpdate>,
    );

    fn run(
        &mut self,
        (bodies, mut turrets, physic_world, update_time, lazy_update): Self::SystemData,
    ) {
        for (turret, body) in (&mut turrets, &bodies).join() {
            turret.remaining_cooldown -= update_time.0;
            if turret.remaining_cooldown <= 0.0 {
                turret.remaining_cooldown = turret.cooldown;
                let mut position = body.get(&physic_world).position().clone();
                position.rotation = ::na::UnitComplex::new(turret.angle);
                ::util::move_forward(&mut position, turret.shoot_distance);
                let bullet = turret.bullet.clone();
                lazy_update.execute(move |world| {
                    bullet.insert(position.into(), world);
                });
            }
        }
    }
}
