pub use vulkano_win::Window;
pub type PhysicWorld = ::nphysics::world::World<f32>;

#[derive(Deref, DerefMut)]
pub struct UpdateTime(pub f32);
pub use animation::AnimationImages;
pub use graphics::Camera;
