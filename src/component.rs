pub use animation::AnimationState;
use nphysics::object::BodyStatus;
use specs::Join;

#[derive(Default)]
pub struct Player;

impl ::specs::Component for Player {
    type Storage = ::specs::NullStorage<Self>;
}

#[derive(Deref, DerefMut)]
pub struct Life(pub usize);

impl ::specs::Component for Life {
    type Storage = ::specs::VecStorage<Self>;
}

#[derive(Default)]
pub struct GravityToPlayers {
    pub mass: f32,
}

impl ::specs::Component for GravityToPlayers {
    type Storage = ::specs::VecStorage<Self>;
}

// Decrease of sound: -6dB
// The sound pressure level (SPL) decreases with doubling of distance by (−)6 dB.
/// This component store the position of the last heard sound
/// Sound is heard at hear_db
pub struct Listener {
    pub hear_position: Option<::na::Vector2<f32>>,
    pub hear_limit: f32,
}
// TODO: better to insert a HeadPosition component so system can iterate on it easily

impl ::specs::Component for Listener {
    type Storage = ::specs::VecStorage<Self>;
}

pub fn play_sound(position: ::na::Vector2<f32>, db: f32, world: &mut ::specs::World) {
    for (listener, body) in (
        &mut world.write::<::component::Listener>(),
        &world.read::<::component::RigidBody>()
    ).join() {
        let listener_position = body.get(&world.read_resource()).position().translation.vector;
        let distance = (position - listener_position).norm();
        if db*(-distance).exp() > listener.hear_limit {
            listener.hear_position = Some(position);
        }
    }
}

// Launch an entitiy
// pub struct Launcher {
//     entity: ::entity::EntitySettings,
//     rate: f32,
//     timer: f32,
// }

#[derive(Deref, DerefMut)]
pub struct Aim(pub f32);

impl ::specs::Component for Aim {
    type Storage = ::specs::VecStorage<Self>;
}

// How to store weapons and allow to switch from them
// and use a trait Weapon
// or an enum
// with a trait you can store it in an inventory

pub struct Weapon {}

#[derive(Clone)]
pub struct RigidBody(::nphysics::object::BodyHandle);

impl ::specs::Component for RigidBody {
    type Storage = ::specs::TrackedStorage<Self, ::specs::DenseVecStorage<Self>>;
}

#[allow(unused)]
impl RigidBody {
    pub fn safe_insert<'a>(
        entity: ::specs::Entity,
        position: ::npm::Isometry<f32>,
        local_inertia: ::npm::Inertia<f32>,
        status: BodyStatus,
        bodies_handle: &mut ::specs::WriteStorage<'a, ::component::RigidBody>,
        physic_world: &mut ::resource::PhysicWorld,
    ) -> ::nphysics::object::BodyHandle {
        let body_handle = physic_world.add_rigid_body(position, local_inertia);
        physic_world.rigid_body_mut(body_handle).unwrap().set_status(status);
        bodies_handle.insert(entity, RigidBody(body_handle));
        body_handle
    }

    #[inline]
    pub fn get<'a>(
        &'a self,
        physic_world: &'a ::resource::PhysicWorld,
    ) -> &'a ::nphysics::object::RigidBody<f32> {
        physic_world
            .rigid_body(self.0)
            .expect("Rigid body in specs does not exist in physic world")
    }

    #[inline]
    pub fn get_mut<'a>(
        &'a mut self,
        physic_world: &'a mut ::resource::PhysicWorld,
    ) -> &'a mut ::nphysics::object::RigidBody<f32> {
        physic_world
            .rigid_body_mut(self.0)
            .expect("Rigid body in specs does not exist in physic world")
    }
}
