use std::collections::HashMap;

pub struct Life(pub usize);

// G*ma*mb/d^2
// to every entities around ?
// or with filter
pub struct Gravity;

// Launch an entitiy
// pub struct Launcher {
//     entity: ::entity::EntitySettings,
//     rate: f32,
//     timer: f32,
// }

pub struct Aim(pub f32);

// How to store weapons and allow to switch from them
// and use a trait Weapon
// or an enum
// with a trait you can store it in an inventory

pub struct Weapon {
}
