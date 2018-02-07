pub use animation::AnimationState;
use nphysics::object::BodyStatus;

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

// G*ma*mb/d^2
// to every entities around ?
// or with filter
pub struct Gravity;

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
