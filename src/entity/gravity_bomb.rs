use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use specs::World;

#[derive(Serialize, Deserialize)]
pub struct GravityBomb {
    animation_specie: AnimationSpecie,
    damage: usize,
    mass: f32,
    powi: i32,
    players_aim_damping: Option<::util::ClampFunction>,
    radius: f32,
}

impl Insertable for GravityBomb {
    fn insert(&self, position: InsertPosition, world: &mut World) {
        let entity = world
            .create_entity()
            .with(::component::AnimationState::new(
                self.animation_specie,
                AnimationName::Idle,
            ))
            .with(::component::Life(1))
            .with(::component::Bomb {
                damage: self.damage,
            })
            .with(::component::GravityToPlayers {
                mass: self.mass,
                powi: self.powi,
            })
            .build();

        if let Some(ref players_aim_damping) = self.players_aim_damping {
            world.write::<::component::PlayersAimDamping>().insert(
                entity,
                ::component::PlayersAimDamping {
                    processor: players_aim_damping.clone(),
                },
            );
        }

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
            body_handle,
            ::na::one(),
            Material::new(0.0, 0.0),
        );
    }
}
