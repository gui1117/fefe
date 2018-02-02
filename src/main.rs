extern crate specs;
extern crate rand;
extern crate ron;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate lyon;
extern crate serde;

mod component;
mod map;
mod entity;
mod resource;
mod animation;

fn main() {
    let mut world = ::specs::World::new();
    if let Err(err) = map::load_map("map".into(), &mut world) {
        println!("{}", err);
    }
}
