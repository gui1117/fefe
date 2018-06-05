#[doc(hidden)]
pub use animation::AnimationState;

use nphysics2d::math::Force;
use nphysics2d::object::BodyStatus;
use ncollide2d::shape::{ShapeHandle, ConvexPolygon};
use retained_storage::RetainedStorage;
use specs::{Component, Entity, NullStorage, VecStorage, WriteStorage};
use std::f32::consts::PI;

#[derive(Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Player;
impl Component for Player {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
pub struct SwordRifle {
    #[serde(skip)]
    pub sword_mode: bool,
    #[serde(skip)]
    pub attack: bool,

    pub sword_damage: usize,
    pub sword_reload_time: f32,
    #[serde(skip)]
    pub sword_reloading: f32,
    pub sword_length: f32,
    pub sword_range: f32,
    #[serde(skip, default = "::util::default_shape_handle")]
    pub sword_shape: ShapeHandle<f32>,

    pub rifle_damage: usize,
    pub rifle_reload_time: f32,
    #[serde(skip)]
    pub rifle_reloading: f32,
}
impl Component for SwordRifle {
    type Storage = VecStorage<Self>;
}
impl SwordRifle {
    pub fn compute_shapes(&mut self) {
        let div = (16.0 * (self.sword_range/(2.0*PI))).ceil() as usize;
        let shape = ConvexPolygon::try_new((0..=div)
            .map(|i| -self.sword_range/2.0 + (i as f32/div as f32)*self.sword_range)
            .map(|angle| ::na::Point2::new(angle.cos(), angle.sin()))
            .chain(Some(::na::Point2::new(0.0, 0.0)))
            .map(|point| self.sword_length*point)
            .collect::<Vec<_>>()
        ).unwrap();
        self.sword_shape = ShapeHandle::new(shape);
    }
}

#[derive(Deserialize, Clone, Deref, DerefMut)]
#[serde(deny_unknown_fields)]
pub struct Aim(pub f32);
impl Component for Aim {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone, Deref, DerefMut)]
#[serde(deny_unknown_fields)]
pub struct Activators(pub Vec<Activator>);
impl Component for Activators {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Activator {
    pub tempo: usize,
    pub partition: Vec<bool>,
    #[serde(skip)]
    pub activated: bool,
}

//////////////////////////////// Life ////////////////////////////////

/// Only against players
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContactDamage(pub usize);
impl Component for ContactDamage {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct DeadOnContact;
impl Component for DeadOnContact {
    type Storage = NullStorage<Self>;
}

#[derive(Deserialize, Clone, Deref, DerefMut)]
#[serde(deny_unknown_fields)]
pub struct Life(pub usize);
impl Component for Life {
    type Storage = VecStorage<Self>;
}

//////////////////////////////// Velocity ////////////////////////////////

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VelocityControl {
    pub velocity: f32,
    #[serde(skip)]
    #[serde(default = "::util::vector_zero")]
    pub direction: ::na::Vector2<f32>,
}
impl Component for VelocityControl {
    type Storage = VecStorage<Self>;
}

pub const VELOCITY_TO_PLAYER_DISTANCE_TO_GOAL: f32 = 0.1;

/// Go to the closest or the last position in memory
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VelocityToPlayerMemory {
    pub activator: usize,
    #[serde(skip)]
    pub last_closest_in_sight: Option<::na::Vector2<f32>>,
    pub velocity: f32,
    /// If false it is equivalent to go to player in sight
    pub memory: bool,
}
impl Component for VelocityToPlayerMemory {
    type Storage = VecStorage<Self>;
}

/// Go into random directions
/// or closest player in sight depending of proba
///
// TODO: maybe make change of direction random when activated
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VelocityToPlayerRandom {
    pub activator: usize,
    /// If some then a random direction with f32 norm is added
    pub random_weighted: Option<f32>,
    /// Clamp the proba with distance to characters
    pub dist_proba_clamp: ::util::ClampFunction,
    /// Clamp the proba with aim of the characters
    pub aim_proba_clamp: ::util::ClampFunction,
    pub velocity: f32,
    pub toward_player: bool,
    #[serde(skip)]
    #[serde(default = "::util::vector_zero")]
    pub current_direction: ::na::Vector2<f32>,
}
impl Component for VelocityToPlayerRandom {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct VelocityToPlayerCircle {
    pub activator: usize,
    pub circle_velocity: f32,
    pub direct_velocity: f32,
    /// Normal distribution
    pub shift_time: (f64, f64),
    #[serde(skip)]
    pub next_shift: f32,
    #[serde(default)]
    pub dir_shift: bool,
}
impl Component for VelocityToPlayerCircle {
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

/// The processor takes distance with player aim in radiant
/// The velocity is multiplied by the result
#[derive(Deserialize, Clone, Deref)]
#[serde(deny_unknown_fields)]
pub struct VelocityDistanceDamping(pub ::util::ClampFunction);
impl Component for VelocityDistanceDamping {
    type Storage = VecStorage<Self>;
}

/// The processor takes distance with player
/// The velocity is multiplied by the result
#[derive(Deserialize, Clone, Deref)]
#[serde(deny_unknown_fields)]
pub struct VelocityAimDamping(pub ::util::ClampFunction);
impl Component for VelocityAimDamping {
    type Storage = VecStorage<Self>;
}

//////////////////////////////// Force ////////////////////////////////

/// The processor takes distance with player aim in radiant
/// The final damping is divided by the result
#[derive(Deserialize, Clone, Deref)]
#[serde(deny_unknown_fields)]
pub struct PlayersAimDamping(pub ::util::ClampFunction);
impl Component for PlayersAimDamping {
    type Storage = VecStorage<Self>;
}

/// The processor takes distance with player
/// The final damping is divided by the result
#[derive(Deserialize, Clone, Deref)]
#[serde(deny_unknown_fields)]
pub struct PlayersDistanceDamping(pub ::util::ClampFunction);
impl Component for PlayersDistanceDamping {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GravityToPlayers {
    pub force: f32,
    pub powi: i32,
}
impl Component for GravityToPlayers {
    type Storage = VecStorage<Self>;
}

pub struct ControlForce(pub Force<f32>);
impl Component for ControlForce {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Damping {
    pub linear: f32,
    pub angular: f32,
}
impl Component for Damping {
    type Storage = VecStorage<Self>;
}

//////////////////////////////// Spawner ////////////////////////////////

/// Spawn an entity if character is in aim at a certain probability function of
/// the distance to the character every time activated
#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct UniqueSpawner {
    pub activator: usize,
    pub spawn: String,
    /// Clamp the proba with distance to characters
    pub dist_proba_clamp: Option<::util::ClampFunction>,
    /// Clamp the proba with aim of the characters
    pub aim_proba_clamp: Option<::util::ClampFunction>,
}
impl Component for UniqueSpawner {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone, Deref, DerefMut)]
#[serde(deny_unknown_fields)]
pub struct TurretSpawner(pub Vec<TurretPart>);
impl Component for TurretSpawner {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TurretPart {
    pub activator: usize,
    pub spawn: String,
    pub rotation_time: usize,
    pub clockwise: bool,
    // TODO: maybe use a vec here so it can have multiple canon
    //       or maybe not
    pub start_time: usize,
    pub shoot_distance: f32,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChamanSpawner {
    pub activator: usize,
    pub spawn: String,
    pub number_of_spawn: usize,
    #[serde(skip)]
    pub spawned: Vec<Entity>,
}
impl Component for ChamanSpawner {
    type Storage = VecStorage<Self>;
}

//////////////////////////////// Physic ////////////////////////////////

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

//////////////////////////////// Debug ////////////////////////////////

#[derive(Deserialize, Clone, Deref, DerefMut)]
#[serde(deny_unknown_fields)]
pub struct DebugCircles(pub Vec<f32>);
impl Component for DebugCircles {
    type Storage = VecStorage<Self>;
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DebugColor(pub usize);
impl Component for DebugColor {
    type Storage = VecStorage<Self>;
}

//////////////////////////////// TODO ////////////////////////////////

// TODO: for bullet
//       position is function of t along an axis ?
// pub struct Positionned {}

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
