use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use rand::{thread_rng, Rand};
use specs::{World, Entity};

#[derive(Deserialize, Clone)]
pub struct Bee {
    pub animation_specie: AnimationSpecie,
    pub radius: f32,
    pub life: usize,
    pub density: f32,
    pub damage: usize,
    pub circle_velocity: f32,
    pub direct_velocity: f32,
    pub shift_time: (f64, f64),
    pub dir_shift: Option<bool>,
    pub dist_damping: Option<::util::ClampFunction>,
    pub aim_damping: Option<::util::ClampFunction>,
}

impl Insertable for Bee {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let entity = world.entities().create();
        world.write().insert(entity, ::component::AnimationState::new(
            self.animation_specie,
            AnimationName::Idle,
        ));
        println!("toto");
        world.write().insert(entity, ::component::VelocityToPlayerCircle {
            circle_velocity: self.circle_velocity,
            direct_velocity: self.direct_velocity,
            dir_shift: self.dir_shift.unwrap_or(bool::rand(&mut thread_rng())),
            next_shift: 0.0,
            shift_time: self.shift_time,
        });
        world.write().insert(entity, ::component::ContactDamage(self.damage));
        world.write().insert(entity, ::component::DeadOnContact);
        world.write().insert(entity, ::component::Contactor(vec![]));
        world.write().insert(entity, ::component::Life(self.life));
        world.write().insert(entity, ::component::DebugColor(3));

        let mut debug_circles = vec![];
        if let Some(dist_damping) = self.dist_damping.clone() {
            debug_circles.push(dist_damping.min_t);
            debug_circles.push(dist_damping.max_t);
            world.write().insert(entity, ::component::VelocityDistanceDamping(dist_damping));
        }
        world.write().insert(entity, ::component::DebugCircles(debug_circles));

        if let Some(aim_damping) = self.aim_damping.clone() {
            world.write().insert(entity, ::component::VelocityAimDamping(aim_damping));
        }

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position.0,
            shape.inertia(self.density),
            shape.center_of_mass(),
            BodyStatus::Dynamic,
            &mut world.write(),
            &mut physic_world,
            &mut world.write_resource(),
        );

        let collider = physic_world.add_collider(
            0.0,
            shape,
            body_handle.0,
            ::na::one(),
            Material::new(0.0, 0.0),
        );
        let mut groups = ::ncollide2d::world::CollisionGroups::new();
        groups.set_membership(&[super::Group::Monster as usize]);
        physic_world.collision_world_mut().set_collision_groups(collider, groups);

        entity
    }
}
