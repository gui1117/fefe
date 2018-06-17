use entity::{FillableObject, InsertableObject, SegmentableObject};
use fnv::FnvHashMap;
use ncollide2d::shape::ShapeHandle;
use nphysics2d::object::BodyHandle;
use specs::Entity;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use app_dirs2::{AppInfo, AppDataType, app_root};

pub use imgui::ImGui;
pub use specs::world::EntitiesRes;
pub use specs::world::LazyUpdate;
pub use audio::Audio;

const APP_INFO: AppInfo = AppInfo { name: "fefe", author: "thiolliere" };
const FILENAME: &str = "save.ron";

lazy_static! {
    static ref SAVE_PATH: PathBuf = {
        let mut path = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        path.push(FILENAME);
        path
    };
}

#[derive(Deserialize, Serialize)]
pub struct Save {
    pub effect_volume: f32,
}

impl Save {
    pub fn load() -> Self {
        File::open(SAVE_PATH.as_path()).ok()
            .and_then(|file| ::ron::de::from_reader(file).ok())
            .unwrap_or(Save {
                effect_volume: 1.0,
            })
    }
}

pub struct WindowSize(pub (u32, u32));

pub type InsertablesMap = HashMap<String, InsertableObject>;

#[derive(Deref, DerefMut)]
pub struct Tempos(pub Vec<Tempo>);
pub struct Tempo {
    pub time: f32,
    pub next_beat_time: f32,
    pub beat: usize,
}

impl Tempo {
    pub fn new(time: f32) -> Self {
        Tempo {
            time,
            next_beat_time: 0.0,
            beat: 0,
        }
    }
}

#[derive(Deref, DerefMut)]
pub struct DebugShapes(pub Vec<(::na::Isometry2<f32>, ShapeHandle<f32>)>);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conf {
    pub fps: usize,
    pub physic_max_timestep: f32,
    pub physic_min_timestep: f32,
    pub zoom: f32,

    pub audio_ear_distance: f32,
    pub audio_z_distance: f32,

    pub insertables: HashMap<String, InsertableObject>,
    pub fillables: HashMap<String, FillableObject>,
    pub segmentables: HashMap<String, SegmentableObject>,
}

impl Conf {
    pub(crate) fn load() -> Self {
        ::ron::de::from_reader(File::open("data/configuration.ron").unwrap()).unwrap()
    }
}

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
