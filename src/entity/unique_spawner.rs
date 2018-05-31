use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable, InsertableObject};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use specs::{World, Entity};

#[derive(Deserialize, Clone)]
pub struct UniqueSpawner {
    pub entity: InsertableObject,
    pub animation_specie: AnimationSpecie,
    pub radius: f32,
    pub dist_proba_clamp: Option<::util::ClampFunction>,
    pub aim_proba_clamp: Option<::util::ClampFunction>,
}

impl Insertable for UniqueSpawner {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let entity = world.entities().create();
        world.write().insert(entity, ::component::AnimationState::new(
            self.animation_specie,
            AnimationName::Idle,
        ));
        world.write().insert(entity, ::component::UniqueSpawner::new(self.entity.clone(), self.dist_proba_clamp.clone(), self.aim_proba_clamp.clone()));
        world.write().insert(entity, ::component::DebugColor(6));

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
        if let Some(ref dist_proba_clamp) = self.dist_proba_clamp {
            world.write().insert(entity, ::component::DebugCircles(vec![
                dist_proba_clamp.min_t,
                dist_proba_clamp.max_t,
            ]));
        }

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position.0,
            shape.inertia(1.0),
            shape.center_of_mass(),
            BodyStatus::Static,
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
