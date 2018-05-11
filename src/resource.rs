use fnv::FnvHashMap;
use nphysics2d::object::BodyHandle;
use specs::Entity;

pub type PhysicWorld = ::nphysics2d::world::World<f32>;

#[derive(Deref, DerefMut)]
pub struct UpdateTime(pub f32);
pub use animation::AnimationImages;
pub use graphics::Camera;

pub struct StepForces(usize);

impl StepForces {
    pub fn new(world: &mut PhysicWorld) -> Self {
        let handle = world.add_force_generator(::force_generator::StepForces::new());
        StepForces(handle)
    }

    #[allow(unused)]
    pub fn get<'a>(&self, world: &'a ::resource::PhysicWorld) -> &'a ::force_generator::StepForces {
        world.force_generator(self.0).downcast_ref().unwrap()
    }

    pub fn get_mut<'a>(
        &self,
        world: &'a mut ::resource::PhysicWorld,
    ) -> &'a mut ::force_generator::StepForces {
        world.force_generator_mut(self.0).downcast_mut().unwrap()
    }
}

#[derive(Deref, DerefMut)]
pub struct BodiesMap(FnvHashMap<BodyHandle, Entity>);

impl BodiesMap {
    pub fn new(ground: Entity) -> Self {
        let mut map = FnvHashMap::default();
        map.insert(BodyHandle::ground(), ground);
        BodiesMap(map)
    }
}
