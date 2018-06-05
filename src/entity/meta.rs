use animation::{AnimationName, AnimationSpecie};
use component::*;
use entity::{InsertPosition, Insertable};
use ncollide2d::shape::{Ball, ShapeHandle};
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::volumetric::Volumetric;
use specs::{Entity, World};

macro_rules! component_list {
    ($($v:ident,)*) => (
        #[derive(Deserialize, Clone)]
        #[serde(deny_unknown_fields)]
        pub enum MetaComponent {
            $($v($v),)*
        }

        impl MetaComponent {
            fn insert(self, entity: Entity, world: &World) {
                use self::*;
                match self {
                    $(MetaComponent::$v(component) => {
                        world.write().insert(entity, component);
                    },)*
                }
            }
        }
    )
}

component_list!{
    Player,
    Aim,
    ContactDamage,
    DeadOnContact,
    Life,
    VelocityControl,
    VelocityToPlayerMemory,
    VelocityToPlayerRandom,
    VelocityToPlayerCircle,
    VelocityDistanceDamping,
    VelocityAimDamping,
    PlayersAimDamping,
    PlayersDistanceDamping,
    GravityToPlayers,
    Damping,
    UniqueSpawner,
    ChamanSpawner,
    TurretSpawner,
    DebugColor,
    DebugCircles,
    Activators,
    SwordRifle,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MetaOverride {
    meta: String,
    components: Vec<MetaComponent>,
}

impl Insertable for MetaOverride {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let meta = world.read_resource::<::resource::InsertablesMap>().get(&self.meta).cloned()
            .ok_or(::failure::err_msg(format!("unknown entity: {}", self.meta))).unwrap();
        let entity = meta.insert(position, world);

        for component in &self.components {
            let component = component.clone();
            component.insert(entity, world);
        }
        entity
    }
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub insert_shift: bool,
    pub launch: bool,
    pub animation_specie: AnimationSpecie,
    pub radius: f32,
    pub density: f32,
    #[serde(with = "::util::BodyStatusDef")]
    pub status: BodyStatus,
    pub groups: Vec<super::Group>,
    pub components: Vec<MetaComponent>,
}

impl Insertable for Meta {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let entity = world.entities().create();

        world.write().insert(
            entity,
            ::component::AnimationState::new(self.animation_specie, AnimationName::Idle),
        );

        for component in &self.components {
            let component = component.clone();
            component.insert(entity, world);
        }

        // TODO: debug circles for components

        if self.components.iter().any(|c| match c {
            MetaComponent::ContactDamage(_) | MetaComponent::VelocityToPlayerCircle(_) | MetaComponent::DeadOnContact(_) => true,
            _ => false,
        }) {
            world.write().insert(entity, ::component::Contactor(vec![]));
        }

        if let Some(ref mut sword_rifle) = world.write::<::component::SwordRifle>().get_mut(entity) {
            sword_rifle.compute_shapes();
        }

        let mut position = position.0;
        if self.insert_shift {
            ::util::move_forward(&mut position, self.radius);
        }

        if self.launch {
            if let Some(ref mut control) = world.write::<::component::VelocityControl>().get_mut(entity) {
                let angle = position.rotation.angle();
                control.direction = ::na::Vector2::new(angle.cos(), angle.sin());
            }
        }

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position,
            shape.inertia(self.density),
            shape.center_of_mass(),
            self.status,
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
        groups.set_membership(&self.groups.iter().map(|g| *g as usize).collect::<Vec<_>>());
        physic_world
            .collision_world_mut()
            .set_collision_groups(collider, groups);

        entity
    }
}
