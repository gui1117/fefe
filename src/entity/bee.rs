use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use rand::{thread_rng, Rand};
use specs::{World, Entity};

#[derive(Serialize, Deserialize, Clone)]
pub struct Bee {
    pub animation_specie: AnimationSpecie,
    pub radius: f32,
    pub velocity: f32,
    pub damage: usize,
    pub circle_velocity: f32,
    pub direct_velocity: f32,
}

impl Insertable for Bee {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let entity = world.entities().create();
        world.write().insert(entity, ::component::AnimationState::new(
            self.animation_specie,
            AnimationName::Idle,
        ));
        world.write().insert(entity, ::component::CircleToPlayer {
            circle_velocity: self.circle_velocity,
            direct_velocity: self.direct_velocity,
            dir_shift: bool::rand(&mut thread_rng()),
        });
        world.write().insert(entity, ::component::ContactDamage(self.damage));
        world.write().insert(entity, ::component::DeadOnContact);
        world.write().insert(entity, ::component::Life(1));
        world.write().insert(entity, ::component::DebugColor(3));

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position.0,
            shape.inertia(1.0),
            shape.center_of_mass(),
            BodyStatus::Kinematic,
            &mut world.write(),
            &mut physic_world,
            &mut world.write_resource(),
        );

        physic_world.add_collider(
            0.0,
            shape,
            body_handle.0,
            ::na::one(),
            Material::new(0.0, 0.0),
        );

        entity
    }
}
