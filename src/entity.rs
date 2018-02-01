pub type EntityPosition = ::lyon::path::default::Path;
// use lyon::tessellation::{FillTessellator, VertexBuffers, FillOptions};
// use lyon::tessellation::geometry_builder::simple_builder;
// use ::lyon::svg::path::PathEvent::*;

#[derive(Serialize, Deserialize)]
pub enum Insertable {
    Monster(Box<Monster>),
    Portal(Box<Portal>),
}

#[derive(Serialize, Deserialize)]
pub struct Monster {
    size: f32,
    attack: f32,
}

#[derive(Serialize, Deserialize)]
pub struct Portal {
    inserted: Insertable,
}

// TODO: do it in a macro
impl Insertable {
    pub fn insert(&self, entity: EntityPosition, world: &mut ::specs::World) {
    }
}
