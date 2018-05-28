pub use animation::AnimationState;
use nphysics2d::math::Force;
use nphysics2d::object::BodyStatus;
use rand::thread_rng;
use rand::distributions::{IndependentSample, Range};
use retained_storage::RetainedStorage;
use specs::{Component, Entity, NullStorage, VecStorage, WriteStorage};
use entity::InsertableObject;

#[derive(Default)]
pub struct Player;
impl Component for Player {
    type Storage = NullStorage<Self>;
}

/// Only against players
pub struct ContactDamage(pub usize);
impl Component for ContactDamage {
    type Storage = VecStorage<Self>;
}

#[derive(Default)]
pub struct DeadOnContact;
impl Component for DeadOnContact {
    type Storage = NullStorage<Self>;
}

pub struct ControlForce(pub Force<f32>);
impl Component for ControlForce {
    type Storage = VecStorage<Self>;
}

pub struct Damping {
    pub linear: f32,
    pub angular: f32,
}
impl Component for Damping {
    type Storage = VecStorage<Self>;
}

#[derive(Deref, DerefMut)]
pub struct Life(pub usize);
impl Component for Life {
    type Storage = VecStorage<Self>;
}
impl From<usize> for Life {
    fn from(l: usize) -> Self {
        Life(l)
    }
}

pub const VELOCITY_TO_PLAYER_MEMORY_REFREASH_RATE: f32 = 0.1;
pub const VELOCITY_TO_PLAYER_DISTANCE_TO_GOAL: f32 = 0.1;
// Go to the closest or the last position in memory
pub struct VelocityToPlayerMemory {
    pub next_refreash: f32,
    pub last_closest_in_sight: Option<::na::Vector2<f32>>,
    pub velocity: f32,
    _force_use_constructor: (),
}
impl Component for VelocityToPlayerMemory {
    type Storage = VecStorage<Self>;
}

impl VelocityToPlayerMemory {
    pub fn new(velocity: f32) -> Self {
        let next_refreash = Range::new(0.0, VELOCITY_TO_PLAYER_MEMORY_REFREASH_RATE).ind_sample(&mut thread_rng());
        VelocityToPlayerMemory {
            next_refreash,
            velocity,
            last_closest_in_sight: None,
            _force_use_constructor: (),
        }
    }
}

pub struct GravityToPlayers {
    pub force: f32,
    pub powi: i32,
}
impl Component for GravityToPlayers {
    type Storage = VecStorage<Self>;
}

// TODO?
// pub struct ToPlayerInSight {
//     next_refreash: f32,
//     closest_in_sight: Option<::na::Vector2<f32>>,
//     force: f32,
// }
// impl Component for ToPlayerInSight {
//     type Storage = VecStorage<Self>;
// }

pub struct PlayersAimDamping {
    // The processor takes distance with aim in radiant
    // It should output the damping associated
    //
    // TODO: maybe use a trait if I want to extend from clamp function too much
    pub processor: ::util::ClampFunction,
}
impl Component for PlayersAimDamping {
    type Storage = VecStorage<Self>;
}

pub const UNIQUE_SPAWNER_TIMER: f32 = 0.1;
// Spawn an entity if character is in aim at a certain probability function of
// the distance to the character every UNIQUE_SPAWNER_TIMER seconds
pub struct UniqueSpawner {
    pub entity: InsertableObject,
    // Clamp the distance to characters
    pub proba: ::util::ClampFunction,
    pub next_refreash: f32,
    _force_use_constructor: (),
}
impl Component for UniqueSpawner {
    type Storage = VecStorage<Self>;
}

impl UniqueSpawner {
    pub fn new(entity: InsertableObject, proba: ::util::ClampFunction) -> Self {
        let next_refreash = Range::new(0.0, UNIQUE_SPAWNER_TIMER).ind_sample(&mut thread_rng());
        UniqueSpawner {
            entity,
            proba,
            next_refreash,
            _force_use_constructor: (),
        }
    }
}

pub struct Turret {
    pub bullet: InsertableObject,
    pub cooldown: f32,
    pub remaining_cooldown: f32,
    pub angle: f32,
    pub shoot_distance: f32,
}
impl Component for Turret {
    type Storage = VecStorage<Self>;
}

// TODO: for bullet
//       position is function of t along an axis ?
pub struct Positionned {}

// TODO: do something gravity like ! with inertia
// // Decrease of sound: -6dB
// // The sound pressure level (SPL) decreases with doubling of distance by (−)6 dB.
// /// This component store the position of the last heard sound
// /// compute the main position
// /// heard sound intensity decrease over time
// pub struct EarPositionMemory {
//     heards: Vec<(::na::Vector2<f32>, f32)>,
//     position: ::na::Vector2<f32>,
//     db: f32,
// }
// // TODO: this can be done better with just position and db and updated each time by decreasing the
// // memoy and add new heards
// // MAYBE: impl some gravity like for sound:
// // sound create a mass at a point during a frame
// // memory is only consequence of intertia

// impl EarPositionMemory {
//     pub fn add_heard(&mut self, heard_position: ::na::Vector2<f32>, heard_db: f32) {
//         self.heards.push((heard_position, heard_db));
//         self.recompute();
//     }

//     pub fn recompute(&mut self) {
//         let (position_sum, db_sum) = self.heards.iter()
//             // FIXME: This mean may have to be on sound pressure instead of dB
//             .fold((::na::zero(), 0.0), |acc: (::na::Vector2<f32>, f32), &(position, db)| (acc.0+position*db, acc.1+db));
//         self.position = position_sum/db_sum;
//         self.db = db_sum;
//     }
// }

// impl Component for EarPositionMemory {
//     type Storage = VecStorage<Self>;
// }

// // TODO: or better have a resource with a channel for send and one for receive
// pub fn play_sound(position: ::na::Vector2<f32>, db: f32, world: &mut World) {
//     for (listener, body) in (
//         &mut world.write::<::component::EarPositionMemory>(),
//         &world.read::<::component::RigidBody>()
//     ).join() {
//         // TODO: computation with a resource for gas constant ?
//         // let listener_position = body.get(&world.read_resource()).position().translation.vector;
//         // let distance = (position - listener_position).norm();
//         // if db*(-distance).exp() > listener.hear_limit {
//         //     listener.hear_position = Some(position);
//         // }
//     }
// }

#[derive(Deref, DerefMut)]
pub struct Aim(pub f32);
impl Component for Aim {
    type Storage = VecStorage<Self>;
}

// How to store weapons and allow to switch from them
// and use a trait Weapon
// or an enum
// with a trait you can store it in an inventory
pub struct Weapon {}

#[derive(Clone)]
pub struct RigidBody(pub ::nphysics2d::object::BodyHandle);
impl Component for RigidBody {
    type Storage = RetainedStorage<Self, VecStorage<Self>>;
}

#[allow(unused)]
impl RigidBody {
    pub fn safe_insert<'a>(
        entity: Entity,
        position: ::nphysics2d::math::Isometry<f32>,
        local_inertia: ::nphysics2d::math::Inertia<f32>,
        local_center_of_mass: ::nphysics2d::math::Point<f32>,
        status: BodyStatus,
        bodies_handle: &mut WriteStorage<'a, ::component::RigidBody>,
        physic_world: &mut ::resource::PhysicWorld,
        bodies_map: &mut ::resource::BodiesMap,
    ) -> Self {
        let body_handle =
            physic_world.add_rigid_body(position, local_inertia, local_center_of_mass);
        {
            let mut rigid_body = physic_world.rigid_body_mut(body_handle).unwrap();
            rigid_body.set_status(status);
            rigid_body
                .activation_status_mut()
                .set_deactivation_threshold(None);
        }
        bodies_map.insert(body_handle, entity);

        bodies_handle.insert(entity, RigidBody(body_handle));
        RigidBody(body_handle)
    }

    #[inline]
    #[allow(unused)]
    pub fn get<'a>(
        &'a self,
        physic_world: &'a ::resource::PhysicWorld,
    ) -> &'a ::nphysics2d::object::RigidBody<f32> {
        physic_world
            .rigid_body(self.0)
            .expect("Rigid body in specs does not exist in physic world")
    }

    #[inline]
    pub fn get_mut<'a>(
        &self,
        physic_world: &'a mut ::resource::PhysicWorld,
    ) -> &'a mut ::nphysics2d::object::RigidBody<f32> {
        physic_world
            .rigid_body_mut(self.0)
            .expect("Rigid body in specs does not exist in physic world")
    }
}

#[derive(Default)]
pub struct Ground;
impl Component for Ground {
    type Storage = NullStorage<Self>;
}

#[derive(Deref, DerefMut)]
pub struct Contactor(pub Vec<Entity>);
impl Component for Contactor {
    type Storage = VecStorage<Self>;
}

pub struct DebugColor(pub usize);
impl Component for DebugColor {
    type Storage = VecStorage<Self>;
}

/// Go into random directions
/// or closest player in sight depending of proba
#[derive(Serialize, Deserialize, Clone)]
pub struct VelocityToPlayerRandom {
    /// If some then the a random direction with f32 norm is added
    pub random_weighted: Option<f32>,
    pub proba: ::util::ClampFunction,
    /// Normal distribution
    pub refreash_time: (f64, f64),
    pub velocity: f32,
    pub toward_player: bool,
    pub next_refreash: f32,
}
impl Component for VelocityToPlayerRandom {
    type Storage = VecStorage<Self>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ChamanSpawnerConf {
    pub entity: InsertableObject,
    pub spawn_time: (f64, f64),
    pub number_of_spawn: usize,
}
impl Into<ChamanSpawner> for ChamanSpawnerConf {
    fn into(self) -> ChamanSpawner {
        ChamanSpawner {
            entity: self.entity,
            spawn_time: self.spawn_time,
            number_of_spawn: self.number_of_spawn,
            spawned: vec![],
            next_spawn: None,
        }
    }
}

pub struct ChamanSpawner {
    pub entity: InsertableObject,
    /// Normal distribution
    pub spawn_time: (f64, f64),
    pub number_of_spawn: usize,
    pub spawned: Vec<Entity>,
    pub next_spawn: Option<f32>,
}
impl Component for ChamanSpawner {
    type Storage = VecStorage<Self>;
}

pub struct CircleToPlayer {
    pub circle_velocity: f32,
    pub direct_velocity: f32,
    pub dir_shift: bool,
}
impl Component for CircleToPlayer {
    type Storage = VecStorage<Self>;
}

// TODO: peut être faire qu'il se repoussent un peu
// peut être aussi faire que pas mis a jour tout le temps
// c'est peut être plus compliqué de faire un truc bien
// mais si on les fait apparaitre dans un cadre autour du héros
// et on les tue si il sorte du cadre ca peut faire un truc
// bien dans une plaine
pub struct Boid {
    pub id: usize,
    pub clamp: ::util::ClampFunction,
    pub velocity: f32,
    pub weight: f32,
}
impl Component for Boid {
    type Storage = VecStorage<Self>;
}
