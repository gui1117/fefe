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
                        world.write_storage().insert(entity, component).unwrap();
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
    Activators,
    SwordRifle,
    PositionInPath,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct MetaOverride {
    meta: String,
    components: Vec<MetaComponent>,
}

impl Insertable for MetaOverride {
    fn insert(&self, position: InsertPosition, world: &World) -> Entity {
        let meta = world
            .read_resource::<::resource::InsertablesMap>()
            .get(&self.meta)
            .cloned()
            .ok_or(::failure::err_msg(format!("unknown entity: {}", self.meta)))
            .unwrap();

        match meta {
            // FIXME: does this override really works for specs storage ?
            super::InsertableObject::Meta(mut meta) => {
                meta.components.extend(self.components.iter().cloned());
                meta.insert(position, world)
            }
            _ => panic!("MetaOverride must override a Meta"),
        }
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
    fn insert(&self, mut position: InsertPosition, world: &World) -> Entity {
        let entity = world.entities().create();

        world
            .write_storage()
            .insert(
                entity,
                ::component::AnimationState::new(self.animation_specie, AnimationName::Idle),
            )
            .unwrap();

        for component in &self.components {
            let component = component.clone();
            component.insert(entity, world);
        }

        // TODO: debug circles for components

        if self.components.iter().any(|c| match c {
            MetaComponent::ContactDamage(_)
            | MetaComponent::VelocityToPlayerCircle(_)
            | MetaComponent::DeadOnContact(_) => true,
            _ => false,
        }) {
            world
                .write_storage()
                .insert(entity, ::component::Contactor(vec![]))
                .unwrap();
        }

        // TODO: rays
        // let mut rays = vec![];
        // if !rays.is_empty() {
        //     world.write_storage().insert(entity, ::component::DebugRays(rays));
        // }

        if let Some(ref mut sword_rifle) = world
            .write_storage::<::component::SwordRifle>()
            .get_mut(entity)
        {
            sword_rifle.compute_shapes();
        }

        if let Some(ref mut position_in_path) = world
            .write_storage::<::component::PositionInPath>()
            .get_mut(entity)
        {
            let path = position.path
                .expect("entity with position in path requires to be inserted with path");
            position_in_path.set(path)
        }

        if self.insert_shift {
            ::util::move_forward(&mut position.position, self.radius);
        }

        if self.launch {
            if let Some(ref mut control) = world
                .write_storage::<::component::VelocityControl>()
                .get_mut(entity)
            {
                let angle = position.position.rotation.angle();
                control.direction = ::na::Vector2::new(angle.cos(), angle.sin());
            }
        }

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(self.radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position.position,
            shape.inertia(self.density),
            shape.center_of_mass(),
            self.status,
            &mut world.write_storage(),
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
