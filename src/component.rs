pub use animation::AnimationState;
use nphysics2d::math::Force;
use nphysics2d::object::BodyStatus;
use retained_storage::RetainedStorage;
use specs::{Component, Entity, NullStorage, VecStorage, WriteStorage};
use entity::InsertableObject;

#[derive(Default)]
pub struct Player;

impl Component for Player {
    type Storage = NullStorage<Self>;
}

pub struct Bomb {
    pub damage: usize,
}
impl Component for Bomb {
    type Storage = VecStorage<Self>;
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

pub struct GravityToPlayers {
    pub mass: f32,
    pub powi: i32,
}

impl Component for GravityToPlayers {
    type Storage = VecStorage<Self>;
}

// TODO: or maybe not
// If multiple players then the closest in sight
pub struct ToPlayerInSight {
    refreash_rate: f32,
    last_refreash: f32,
    closest_in_sight: Option<::na::Vector2<f32>>,
    force: f32,
}

impl Component for ToPlayerInSight {
    type Storage = VecStorage<Self>;
}

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
