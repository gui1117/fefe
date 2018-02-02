// use lyon::tessellation::{FillTessellator, VertexBuffers, FillOptions};
// use lyon::tessellation::geometry_builder::simple_builder;
// use ::lyon::svg::path::PathEvent::*;
use ::lyon::svg::path::default::Path;

pub struct InsertPosition([f32; 2], f32);

impl ::map::TryFromPath for InsertPosition {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error> {
        // TODO:
        Ok(InsertPosition([1.0, 1.0], 1.0))
    }
}

impl ::map::Builder for InsertableObject {
    type Position = InsertPosition;
    fn build(&self, position: Self::Position, world: &mut ::specs::World) {
        self.insert(position, world);
    }
}

pub trait Insertable {
    fn insert(&self, position: InsertPosition, world: &mut ::specs::World);
}

#[derive(Serialize, Deserialize)]
pub enum InsertableObject {
    Monster(Box<Monster>),
    Portal(Box<Portal>),
}

impl Insertable for InsertableObject {
    fn insert(&self, position: InsertPosition, world: &mut ::specs::World) {
        //TODO
    }
}

#[derive(Serialize, Deserialize)]
pub struct Monster {
    size: f32,
    attack: f32,
}

impl Insertable for Monster {
    fn insert(&self, position: InsertPosition, world: &mut ::specs::World) {
        //TODO
    }
}

#[derive(Serialize, Deserialize)]
pub struct Portal {
    inserted: InsertableObject,
}

impl Insertable for Portal {
    fn insert(&self, position: InsertPosition, world: &mut ::specs::World) {
        //TODO
    }
}
