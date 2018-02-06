// use lyon::tessellation::{FillTessellator, VertexBuffers, FillOptions};
// use lyon::tessellation::geometry_builder::simple_builder;
// use ::lyon::svg::path::PathEvent::*;
use ::lyon::svg::path::default::Path;
use ::lyon::svg::path::PathEvent;

#[derive(Deref, DerefMut)]
pub struct InsertPosition(::na::Isometry2<f32>);

impl ::map::TryFromPath for InsertPosition {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error> {
        let mut path_iter = value.path_iter();
        match (path_iter.next(), path_iter.next()) {
            (Some(PathEvent::MoveTo(p1)), Some(PathEvent::LineTo(p2))) => {
                let p1 = ::na::Vector2::new(p1.x, p1.y);
                let p2 = ::na::Vector2::new(p2.x, p2.y);
                let dir = p2-p1;
                Ok(InsertPosition(::na::Isometry2::new(p1, dir[1].atan2(dir[0]))))
            }
            _ => Err(format_err!("invalid path for InsertPosition.
After being converted to absolute event path must be:
[MoveTo(_), LineTo(_)]
or it is:
{:?}", value.path_iter().collect::<Vec<_>>()))
        }
    }
}

impl ::map::Builder for InsertableObject {
    type Position = InsertPosition;
    fn build(&self, position: Self::Position, world: &mut ::specs::World) {
        self.insert(position, world);
    }
}

macro_rules! object {
    (
        $t:ident, $f:ident, $p:ident, $o:ident {
            $($v:ident)*,
        }
    ) => (
        pub trait $t {
            fn $f(&self, position: $p, world: &mut ::specs::World);
        }

        #[derive(Serialize, Deserialize)]
        pub enum $o {
            $($v(Box<$v>))*,
        }

        impl $t for $o {
            fn $f(&self, position: $p, world: &mut ::specs::World) {
                match self {
                    $(&$o::$v(ref p) => p.insert(position, world)),*
                }
            }
        }
    );
}

object!(Insertable, insert, InsertPosition, InsertableObject {
    Player,
});

#[derive(Serialize, Deserialize)]
pub struct Player;

impl Insertable for Player {
    fn insert(&self, position: InsertPosition, world: &mut ::specs::World) {
        let p = world.create_entity()
            .with(::component::AnimationState::new(
                ::animation::AnimationSpecie::Character,
                ::animation::AnimationName::IdleRifle,
            ))
            .build();

        ::component::RigidBody::safe_insert(
            p, position.0, ::npm::Inertia::zero(),
            &mut world.write(),
            &mut world.write_resource(),
        );
    }
}
