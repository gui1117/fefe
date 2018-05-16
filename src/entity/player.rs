use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::math::Force;
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use specs::World;

#[derive(Serialize, Deserialize, Clone)]
pub struct Player;

impl Insertable for Player {
    fn insert(&self, position: InsertPosition, world: &World) {
        let conf = world.read_resource::<::resource::Conf>();
        let entity = world.entities().create();

        world.write().insert(entity, ::component::AnimationState::new(
            AnimationSpecie::Character,
            AnimationName::IdleRifle,
        ));
        world.write().insert(entity, ::component::Player);
        world.write().insert(entity, ::component::Aim(position.rotation.angle()));
        world.write().insert(entity, ::component::Life(1));
        world.write().insert(entity, ::component::ControlForce(Force::zero()));
        world.write().insert(entity, ::component::Damping {
            linear: conf.player_linear_damping,
            angular: conf.player_angular_damping,
        });

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(conf.player_radius));
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
    }
}
