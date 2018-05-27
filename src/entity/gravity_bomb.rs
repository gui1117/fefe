use animation::{AnimationName, AnimationSpecie};
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use specs::World;

#[derive(Serialize, Deserialize, Clone)]
pub struct GravityBomb {
    pub animation_specie: AnimationSpecie,
    pub damage: usize,
    pub force: f32,
    pub powi: i32,
    pub players_aim_damping: Option<::util::ClampFunction>,
    pub radius: f32,
    pub insert_shift: bool,
}

impl Insertable for GravityBomb {
    fn insert(&self, position: InsertPosition, world: &World) {
        let entity = world.entities().create();

        world.write().insert(entity, ::component::AnimationState::new(
            self.animation_specie,
            AnimationName::Idle,
        ));
        world.write().insert(entity, ::component::Life(1));
        world.write().insert(entity, ::component::ContactDamage(self.damage));
        world.write().insert(entity, ::component::DeadOnContact);
        world.write().insert(entity, ::component::GravityToPlayers {
            force: self.force,
            powi: self.powi,
        });
        world.write().insert(entity, ::component::Contactor(vec![]));

        if let Some(ref players_aim_damping) = self.players_aim_damping {
            world.write::<::component::PlayersAimDamping>().insert(
                entity,
                ::component::PlayersAimDamping {
                    processor: players_aim_damping.clone(),
                },
            );
        }

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let mut position = position.0;
        if self.insert_shift {
            ::util::move_forward(&mut position, self.radius);
        }

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body = ::component::RigidBody::safe_insert(
            entity,
            position,
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
            body.0,
            ::na::one(),
            Material::new(0.0, 0.0),
        );
    }
}
