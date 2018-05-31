use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable, InsertableObject};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use specs::{World, Entity};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Turret {
    pub bullet: InsertableObject,
    pub animation_specie: AnimationSpecie,
    pub radius: f32,
    pub max_cooldown: f32,
}

impl Insertable for Turret {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let mut rng = thread_rng();

        let cooldown = Range::new(0.0, self.max_cooldown).ind_sample(&mut rng);
        let start_remaining_cooldown = cooldown;

        let entity = world.entities().create();
        world.write().insert(entity, ::component::AnimationState::new(
            self.animation_specie,
            AnimationName::Idle,
        ));
        world.write().insert(entity, ::component::Turret {
            cooldown,
            bullet: self.bullet.clone(),
            remaining_cooldown: start_remaining_cooldown,
            angle: position.0.rotation.angle(),
            shoot_distance: self.radius,
        });

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position.0,
            shape.inertia(1.0),
            shape.center_of_mass(),
            BodyStatus::Dynamic,
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
