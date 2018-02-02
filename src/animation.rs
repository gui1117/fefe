pub struct AnimationTableConfig(HashMap<(AnimationSpecie, AnimationState), String>);
// TODO: put in config

pub struct AnimationTable(HashMap<(AnimationSpecie, AnimationState), Animation>);
// TODO: put in resource

pub struct Animation {
    framerate: usize,
    images: (),//TODO with engine
}

pub struct AnimationSettings {
    aim: Vec<AnimationPartSettings>,
    position: Vec<AnimationPartSettings>,
}

pub struct AnimationPartSettings {
    file: String,
    layer: f32,
    walk: bool
}

// image, layer, duration
// and also if anchor to position or aim
pub enum AnimationName {
    ShootRifle,
    IdleRifle,
    TakeRifle,
    UntakeRifle,
}

pub enum AnimationSpecie {
    Character,
    Monster,
}

pub struct AnimationState {
    /// 0 is no walk
    // the idle shoulder animation should use walk distance too
    walk_distance: f32,
    specie: AnimationSpecie,
    // Store animation instead of name is feasible but not handy
    // because push animation would require AnimationTable
    // TODO: or maybe it is OK if graphics store image in hashmap of stringname
    idle_animation: AnimationName,
    animations: Vec<AnimationName>,
    timer: f32,
}
