use specs::{Join, Fetch, FetchMut, ReadStorage, System, WriteStorage};
use ncollide2d::world::CollisionGroups;
use ncollide2d::query;

pub struct SwordRifleSystem;

impl<'a> System<'a> for SwordRifleSystem {
    type SystemData = (
        ReadStorage<'a, ::component::RigidBody>,
        WriteStorage<'a, ::component::SwordRifle>,
        WriteStorage<'a, ::component::Life>,
        Fetch<'a, ::resource::PhysicWorld>,
        Fetch<'a, ::resource::BodiesMap>,
        Fetch<'a, ::resource::UpdateTime>,
        FetchMut<'a, ::resource::DebugShapes>,
    );

    fn run(&mut self, (bodies, mut sword_rifles, mut lives, physic_world, bodies_map, update_time, mut debug_shapes): Self::SystemData) {
        for (sr, body) in (&mut sword_rifles, &bodies).join() {
            sr.sword_reloading -= update_time.0;
            if sr.attack {
                let body = body.get(&physic_world);

                let aabb = sr.sword_shape.aabb(&body.position());
                let mut groups = CollisionGroups::new();
                groups.set_whitelist(&[::entity::Group::Monster as usize]);

                let contacts = physic_world.collision_world()
                    .interferences_with_aabb(&aabb, &groups)
                    .filter(|obj| {
                        query::contact(obj.position(), obj.shape().as_ref(), &body.position(), sr.sword_shape.as_ref(), 0.0).is_some()
                    })
                    .map(|obj| bodies_map.get(&obj.data().body()).unwrap());

                for contact in contacts {
                    if let Some(ref mut life) = lives.get_mut(*contact) {
                        life.0 -= sr.sword_damage;
                    }
                }

                debug_shapes.push((
                    body.position(),
                    sr.sword_shape.clone(),
                ));
            }
        }
    }
}
