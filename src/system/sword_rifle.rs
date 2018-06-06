use specs::{Join, ReadExpect, WriteExpect, ReadStorage, System, WriteStorage};
use ncollide2d::world::CollisionGroups;
use ncollide2d::query::{self, Ray};
use entity::Group;

pub struct SwordRifleSystem;

impl<'a> System<'a> for SwordRifleSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        ReadStorage<'a, ::component::Aim>,
        WriteStorage<'a, ::component::SwordRifle>,
        WriteStorage<'a, ::component::Life>,
        ReadExpect<'a, ::resource::PhysicWorld>,
        ReadExpect<'a, ::resource::BodiesMap>,
        ReadExpect<'a, ::resource::UpdateTime>,
        WriteExpect<'a, ::resource::DebugShapes>,
    );

    fn run(&mut self, (bodies, aims, mut sword_rifles, mut lives, physic_world, bodies_map, update_time, mut debug_shapes): Self::SystemData) {
        for (sr, aim, body) in (&mut sword_rifles, &aims, &bodies).join() {
            sr.sword_reloading -= update_time.0;
            sr.rifle_reloading -= update_time.0;

            if sr.attack {
                if sr.sword_mode {
                    if sr.sword_reloading <= 0.0 {
                        sr.sword_reloading = sr.sword_reload_time;
                        let body = body.get(&physic_world);
                        let position = body.position() * ::na::UnitComplex::new(aim.0);

                        let aabb = sr.sword_shape.aabb(&body.position());
                        let mut groups = CollisionGroups::new();
                        groups.set_whitelist(&[::entity::Group::Monster as usize]);

                        let contacts = physic_world.collision_world()
                            .interferences_with_aabb(&aabb, &groups)
                            .filter(|obj| {
                                query::contact(obj.position(), obj.shape().as_ref(), &position, sr.sword_shape.as_ref(), 0.0).is_some()
                            })
                            .map(|obj| bodies_map.get(&obj.data().body()).unwrap());

                        for contact in contacts {
                            if let Some(ref mut life) = lives.get_mut(*contact) {
                                life.0 -= sr.sword_damage;
                            }
                        }

                        debug_shapes.push((
                            position,
                            sr.sword_shape.clone(),
                        ));
                    }
                } else {
                    if sr.rifle_reloading <= 0.0 {
                        sr.rifle_reloading = sr.rifle_reload_time;
                        let body = body.get(&physic_world);

                        let ray = Ray {
                            origin: ::na::Point2::from_coordinates(body.position().translation.vector),
                            dir: ::na::Vector2::new(aim.cos(), aim.sin()),
                        };
                        let mut groups = CollisionGroups::new();
                        groups.set_whitelist(&[Group::Monster as usize, Group::Wall as usize]);

                        let contacts = physic_world.collision_world()
                            .interferences_with_ray(&ray, &groups)
                            .min_by_key(|(_, intersection)| {
                                (intersection.toi * ::CMP_PRECISION) as isize
                            })
                            .map(|(obj, _)| bodies_map.get(&obj.data().body()).unwrap());

                        for contact in contacts {
                            if let Some(ref mut life) = lives.get_mut(*contact) {
                                life.0 -= sr.rifle_damage;
                            }
                        }
                    }
                }
            }
        }
    }
}
