use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use specs::{World, Entity};

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Chaman {
    pub animation_specie: AnimationSpecie,
    pub radius: f32,
    pub life: usize,
    pub density: f32,
    pub velocity_to_player_random: ::component::VelocityToPlayerRandom,
    pub chaman_spawner: ::component::ChamanSpawnerConf,
    pub dist_damping: Option<::util::ClampFunction>,
    pub aim_damping: Option<::util::ClampFunction>,
}

impl Insertable for Chaman {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let entity = world.entities().create();
        world.write().insert(entity, ::component::AnimationState::new(
            self.animation_specie,
            AnimationName::Idle,
        ));
        world.write().insert(entity, self.velocity_to_player_random.clone());
        world.write::<::component::ChamanSpawner>().insert(entity, self.chaman_spawner.clone().into());
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
