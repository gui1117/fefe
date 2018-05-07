pub type PhysicWorld = ::nphysics2d::world::World<f32>;

#[derive(Deref, DerefMut)]
pub struct UpdateTime(pub f32);
pub use animation::AnimationImages;
pub use graphics::Camera;

pub struct CharacterDamping(usize);

impl CharacterDamping {
    pub fn new(world: &mut PhysicWorld) -> Self {
        let force_generator = ::force_generator::Damping::new(
            ::CFG.player_linear_damping,
            ::CFG.player_angular_damping,
        );
        let handle = world.add_force_generator(force_generator);
        CharacterDamping(handle)
    }
}

impl ::force_generator::DerefDamping for CharacterDamping {
    fn force_generator_handle(&self) -> usize {
        self.0
    }
}
