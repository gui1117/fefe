extern crate specs;
extern crate rand;
extern crate ron;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate lyon;
extern crate serde;
#[macro_use]
extern crate lazy_static;

mod component;
mod map;
mod entity;
mod resource;
mod animation;
mod configuration;

pub use configuration::CFG;

fn main() {
    let mut world = ::specs::World::new();
    if let Err(err) = map::load_map("map".into(), &mut world) {
        println!("{}", err);
    }
    let _: animation::AnimationsCfg = ::ron::de::from_str("AnimationsCfg(
    table: {
        (Character, ShootRifle): [\"toto\"]
    },
    parts: {
        \"toto\": (
            filename: \"toto\",
            layer: 0.1,
            framerate: Walk(0.1),
        )
    }
)").unwrap();
}
