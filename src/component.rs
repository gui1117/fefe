pub use animation::AnimationState;
use nphysics2d::object::BodyStatus;
use specs::{Component, VecStorage, Entity, WriteStorage, NullStorage};
use retained_storage::RetainedStorage;

#[derive(Default)]
pub struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

#[derive(Deref, DerefMut)]
pub struct Life(pub usize);

impl Component for Life {
    type Storage = VecStorage<Self>;
}

pub struct GravityToPlayers {
    pub mass: f32,
}

impl Component for GravityToPlayers {
    type Storage = VecStorage<Self>;
}

pub struct ToPlayerInSight;

impl Component for ToPlayerInSight {
    type Storage = VecStorage<Self>;
}

pub struct PlayerAimDamping {
    pub processor: Box<Fn(f32) -> f32 + Sync + Send>,
}

impl Component for PlayerAimDamping {
    type Storage = VecStorage<Self>;
}

pub struct PlayerAimInvDamping {
    pub processor: Box<Fn(f32) -> f32 + Sync + Send>,
}

impl Component for PlayerAimInvDamping {
    type Storage = VecStorage<Self>;
}

#[derive(Deref, DerefMut)]
pub struct Turret(Vec<TurretPart>);

pub struct TurretPart {
    pub bullet: Box<::entity::Insertable + Sync + Send>,
    /// Vec of cooldown
    pub shoots: Vec<f32>,
    /// Time at startup
    pub startup_time: f32,
    pub delta_angle: f32,
}

impl Component for Turret {
    type Storage = VecStorage<Self>;
}

// TODO: for bullet
//       position is function of t along an axis ?
pub struct Positionned {
}

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
    ) -> ::nphysics2d::object::BodyHandle {
        let body_handle = physic_world.add_rigid_body(position, local_inertia, local_center_of_mass);
        physic_world.rigid_body_mut(body_handle).unwrap().set_status(status);
        bodies_handle.insert(entity, RigidBody(body_handle));
        body_handle
    }

    #[inline]
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
        &'a mut self,
        physic_world: &'a mut ::resource::PhysicWorld,
    ) -> &'a mut ::nphysics2d::object::RigidBody<f32> {
        physic_world
            .rigid_body_mut(self.0)
            .expect("Rigid body in specs does not exist in physic world")
    }
}
