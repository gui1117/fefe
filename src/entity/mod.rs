use itertools::Itertools;
use lyon::svg::path::default::Path;
use lyon::svg::path::iterator::PathIterator;
use lyon::svg::path::{FlattenedEvent, PathEvent};
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::{FillOptions, FillTessellator, FillVertex, VertexBuffers};
use specs::{World, Entity};

pub use component::VelocityToPlayerRandom;
pub use component::ChamanSpawnerConf;

#[repr(usize)]
pub enum Group {
    Player,
    Wall,
    Monster,
}

const SEGMENTS_POSITION_FLATTENED_TOLERANCE: f32 = 1.0;

#[derive(Deref, DerefMut)]
#[doc(hidden)]
pub struct SegmentsPosition(Vec<[::na::Point2<f32>; 2]>);

impl ::map::TryFromPath for SegmentsPosition {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error> {
        let path_iter = value
            .path_iter()
            .flattened(SEGMENTS_POSITION_FLATTENED_TOLERANCE);

        let err = |msg: String| {
            format_err!(
                "invalid path for SegmentsPosition.
{}
Full path after being converted to absolute flattened event path:
{:?}",
                msg,
                value
                    .path_iter()
                    .flattened(SEGMENTS_POSITION_FLATTENED_TOLERANCE)
                    .collect::<Vec<_>>()
            )
        };

        let mut last_start = None;
        let mut segments = vec![];
        for (e1, e2) in path_iter.tuple_windows() {
            if let FlattenedEvent::MoveTo(p1) = e1 {
                last_start = Some(p1);
            }
            if let FlattenedEvent::MoveTo(p2) = e2 {
                last_start = Some(p2);
            }

            let option_p1_p2 = match (e1, e2) {
                (FlattenedEvent::MoveTo(p1), FlattenedEvent::LineTo(p2))
                | (FlattenedEvent::LineTo(p1), FlattenedEvent::LineTo(p2)) => Some((p1, p2)),
                (FlattenedEvent::LineTo(p1), FlattenedEvent::Close) => Some((
                    p1,
                    last_start.ok_or_else(|| {
                        err("Closed event without MoveTo event before".to_string())
                    })?,
                )),
                (FlattenedEvent::Close, FlattenedEvent::MoveTo(_)) => None,
                (FlattenedEvent::LineTo(_), FlattenedEvent::MoveTo(_)) => None,
                (FlattenedEvent::Close, FlattenedEvent::LineTo(_)) => {
                    return Err(err("Close event followed by LineTo event".to_string()))
                }
                (FlattenedEvent::MoveTo(_), FlattenedEvent::MoveTo(_)) => {
                    return Err(err("MoveTo event followed by MoveTo event".to_string()))
                }
                (FlattenedEvent::MoveTo(_), FlattenedEvent::Close) => {
                    return Err(err("MoveTo event followed by Close event".to_string()))
                }
                (FlattenedEvent::Close, FlattenedEvent::Close) => {
                    return Err(err("Close event followed by Close event".to_string()))
                }
            };
            if let Some((p1, p2)) = option_p1_p2 {
                segments.push([::na::Point2::new(p1.x, p1.y), ::na::Point2::new(p2.x, p2.y)]);
            }
        }
        Ok(SegmentsPosition(segments))
    }
}

#[derive(Deref, DerefMut)]
#[doc(hidden)]
pub struct FillPosition(Vec<[::na::Point2<f32>; 3]>);

impl ::map::TryFromPath for FillPosition {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error> {
        let mut buffers: VertexBuffers<FillVertex> = VertexBuffers::new();

        {
            let mut vertex_builder = simple_builder(&mut buffers);
            let mut tessellator = FillTessellator::new();

            tessellator
                .tessellate_path(
                    value.path_iter(),
                    &FillOptions::default().with_normals(false),
                    &mut vertex_builder,
                )
                .map_err(|e| {
                    format_err!(
                        "invalid path for FillPosition: {:?} for path: \"{:?}\"",
                        e,
                        value
                    )
                })?;
        }

        let mut indices_iter = buffers.indices.iter();
        let mut position = vec![];
        while let (Some(i1), Some(i2), Some(i3)) = (
            indices_iter.next(),
            indices_iter.next(),
            indices_iter.next(),
        ) {
            let v1 = buffers.vertices[*i1 as usize].position;
            let v2 = buffers.vertices[*i2 as usize].position;
            let v3 = buffers.vertices[*i3 as usize].position;
            position.push([
                ::na::Point2::new(v1.x, v1.y),
                ::na::Point2::new(v2.x, v2.y),
                ::na::Point2::new(v3.x, v3.y),
            ]);
        }
        Ok(FillPosition(position))
    }
}

#[derive(Deref, DerefMut)]
#[doc(hidden)]
pub struct InsertPosition(::na::Isometry2<f32>);

impl ::map::TryFromPath for InsertPosition {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error> {
        let mut path_iter = value.path_iter();
        match (path_iter.next(), path_iter.next()) {
            (Some(PathEvent::MoveTo(p1)), Some(PathEvent::LineTo(p2))) => {
                let p1 = ::na::Vector2::new(p1.x, p1.y);
                let p2 = ::na::Vector2::new(p2.x, p2.y);
                let dir = p2 - p1;
                Ok(InsertPosition(::na::Isometry2::new(
                    p1,
                    dir[1].atan2(dir[0]),
                )))
            }
            _ => Err(format_err!(
                "invalid path for InsertPosition.
After being converted to absolute event path must be:
[MoveTo(_), LineTo(_)]
or it is:
{:?}",
                value.path_iter().collect::<Vec<_>>()
            )),
        }
    }
}

impl From<::na::Isometry2<f32>> for InsertPosition {
    fn from(isometry: ::na::Isometry2<f32>) -> Self {
        InsertPosition(isometry)
    }
}

macro_rules! object {
    (
        $t:ident $(-> $r:path)*, $f:ident, $p:ident, $o:ident {
            $($v:ident),*
        }
    ) => (
        object!($t $(-> $r)*, $f, $p, $o {
            $($v,)*
        });
    );
    (
        $t:ident $(-> $r:path)*, $f:ident, $p:ident, $o:ident {
            $($v:ident,)*
        }
    ) => (
        pub (crate) trait $t {
            fn $f(&self, position: $p, world: &World) $(-> $r)*;
        }

        #[derive(Deserialize, Clone)]
        pub enum $o {
            $($v(Box<$v>),)*
        }

        impl $t for $o {
            fn $f(&self, position: $p, world: &World) $(-> $r)* {
                match self {
                    $(&$o::$v(ref p) => p.$f(position, world)),*
                }
            }
        }

        impl ::map::Builder for $o {
            type Position = $p;
            fn build(&self, position: Self::Position, world: &mut World) {
                self.$f(position, world);
            }
        }
    );
}

object!(
    Insertable -> Entity,
    insert,
    InsertPosition,
    InsertableObject {
        Player,
        Charger,
        UniqueSpawner,
        Walker,
        Chaman,
        Bee,
    }
);

object!(Fillable, fill, FillPosition, FillableObject { Wall });

object!(
    Segmentable,
    segments,
    SegmentsPosition,
    SegmentableObject { Wall }
);

mod gravity_bomb;
pub use self::gravity_bomb::GravityBomb;

mod player;
pub use self::player::Player;

mod wall;
pub use self::wall::Wall;

mod turret;
pub use self::turret::Turret;

mod unique_spawner;
pub use self::unique_spawner::UniqueSpawner;

mod charger;
pub use self::charger::Charger;

mod walker;
pub use self::walker::Walker;

mod chaman;
pub use self::chaman::Chaman;

mod bee;
pub use self::bee::Bee;
