use specs::World;
use retained_storage::Retained;

macro_rules! try_multiple_time {
    ($e:expr) => (
        {
            let mut error_timer = 0;
            let mut res = $e;
            while res.is_err() {
                ::std::thread::sleep(::std::time::Duration::from_millis(100));
                error_timer += 1;
                if error_timer > 10 {
                    break;
                }
                res = $e;
            }
            res
        }
    )
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ClampFunction {
    pub min_value: f32,
    pub max_value: f32,
    pub min_t: f32,
    pub max_t: f32,
}

impl ClampFunction {
    pub fn compute(&self, t: f32) -> f32 {
        debug_assert!(self.min_t < self.max_t);
        if t <= self.min_t {
            self.min_value
        } else if t >= self.max_t {
            self.max_value
        } else {
            (t - self.min_t) / (self.max_t - self.min_t) * (self.max_value - self.min_value)
                + self.min_value
        }
    }
}

pub fn reset_world(world: &mut World) {
    world.maintain();
    world.delete_all();

    let ground = world.create_entity().with(::component::Ground).build();
    world.add_resource(::resource::BodiesMap::new(ground));

    let mut physic_world = ::resource::PhysicWorld::new();
    world.add_resource(::resource::StepForces::new(&mut physic_world));
    world.add_resource(physic_world);
}

pub fn safe_maintain(world: &mut World) {
    world.maintain();
    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
    let retained = world
        .write::<::component::RigidBody>()
        .retained()
        .iter()
        .map(|r| r.0)
        .collect::<Vec<_>>();
    physic_world.remove_bodies(&retained);
}
