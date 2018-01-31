pub type EntityPosition = ::lyon::path::default::Path;
// use lyon::tessellation::{FillTessellator, VertexBuffers, FillOptions};
// use lyon::tessellation::geometry_builder::simple_builder;
// use ::lyon::svg::path::PathEvent::*;

#[derive(Serialize, Deserialize)]
pub enum EntitySettings {
    Monster1 {
        size: f32,
        attack: f32,
    },
    Monster2 {
        size: f32,
        attack: f32,
    },
}

impl EntitySettings {
    pub fn insert(&self, entity: EntityPosition, world: &mut ::specs::World) {
        // Err(format_err!("the following path does not correspond to a valid entity position \"{}\"", commands))
        unimplemented!();
    }
}
