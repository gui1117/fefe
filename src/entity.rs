pub enum EntityPosition {
    Isometry2(()),
}

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
        unimplemented!();
    }
}
