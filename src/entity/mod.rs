use lyon::tessellation::{FillOptions, FillTessellator, FillVertex, VertexBuffers};
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::svg::path::iterator::PathIterator;
use lyon::svg::path::default::Path;
use lyon::svg::path::{FlattenedEvent, PathEvent};
use nphysics2d::volumetric::Volumetric;
use nphysics2d::object::{BodyStatus, Material};
use nphysics2d::math::Force;
use ncollide2d::shape::{Ball, ConvexPolygon, Segment, ShapeHandle};
use specs::World;
use itertools::Itertools;

const SEGMENTS_POSITION_FLATTENED_TOLERANCE: f32 = 1.0;

#[derive(Deref, DerefMut)]
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
                    last_start.ok_or_else(
                        || err("Closed event without MoveTo event before".to_string()),
                    )?,
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

macro_rules! object {
    (
        $t:ident, $f:ident, $p:ident, $o:ident {
            $($v:ident),*
        }
    ) => (
        object!($t, $f, $p, $o {
            $($v,)*
        });
    );
    (
        $t:ident, $f:ident, $p:ident, $o:ident {
            $($v:ident)*,
        }
    ) => (
        pub trait $t {
            fn $f(&self, position: $p, world: &mut World);
        }

        #[derive(Serialize, Deserialize)]
        pub enum $o {
            $($v(Box<$v>))*,
        }

        impl $t for $o {
            fn $f(&self, position: $p, world: &mut World) {
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
    Insertable,
    insert,
    InsertPosition,
    InsertableObject { Player }
);

object!(Fillable, fill, FillPosition, FillableObject { Wall });

object!(
    Segmentable,
    segments,
    SegmentsPosition,
    SegmentableObject { Wall }
);

#[derive(Serialize, Deserialize)]
pub struct Wall;

impl Fillable for Wall {
    fn fill(&self, position: FillPosition, world: &mut World) {
        // FIXME: is it better if we center the triangle on the position of the collider or on the
        // position of the rigid body ?
        // also it is better if we have one rigid body per collider ?
        //
        // we probably don't care as it is static
        //
        // TODO: mabye we should use the ground !!!
        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let body_handle = physic_world.add_rigid_body(
            ::na::Isometry2::new(::na::Vector2::new(0.0, 0.0), 0.0),
            ::nphysics2d::math::Inertia::zero(),
            ::nphysics2d::math::Point::new(0.0, 0.0),
        );
        physic_world
            .rigid_body_mut(body_handle)
            .unwrap()
            .set_status(BodyStatus::Static);

        for position in position.iter() {
            physic_world.add_collider(
                0.0,
                ShapeHandle::new(
                    ConvexPolygon::try_new(position.iter().cloned().collect()).unwrap(),
                ),
                body_handle,
                ::na::one(),
                Material::new(0.0, 0.0),
            );
        }
    }
}

impl Segmentable for Wall {
    fn segments(&self, position: SegmentsPosition, world: &mut World) {
        // FIXME: same fill
        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let body_handle = physic_world.add_rigid_body(
            ::na::Isometry2::new(::na::Vector2::new(0.0, 0.0), 0.0),
            ::nphysics2d::math::Inertia::zero(),
            ::nphysics2d::math::Point::new(0.0, 0.0),
        );
        physic_world
            .rigid_body_mut(body_handle)
            .unwrap()
            .set_status(BodyStatus::Static);

        for position in position.iter() {
            physic_world.add_collider(
                0.0,
                ShapeHandle::new(Segment::from_array(&position).clone()),
                body_handle,
                ::na::one(),
                Material::new(0.0, 0.0),
            );
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Player;

impl Insertable for Player {
    fn insert(&self, position: InsertPosition, world: &mut World) {
        let entity = world
            .create_entity()
            .with(::component::AnimationState::new(
                ::animation::AnimationSpecie::Character,
                ::animation::AnimationName::IdleRifle,
            ))
            .with(::component::Player)
            .with(::component::Aim(position.rotation.angle()))
            .with(::component::Life(1))
            .with(::component::ControlForce(Force::zero()))
            .with(::component::Damping {
                linear: ::CFG.player_linear_damping,
                angular: ::CFG.player_angular_damping,
            })
            .build();

        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        let shape = ShapeHandle::new(Ball::new(::CFG.player_radius));
        let body_handle = ::component::RigidBody::safe_insert(
            entity,
            position.0,
            shape.inertia(1.0),
            shape.center_of_mass(),
            BodyStatus::Dynamic,
            &mut world.write(),
            &mut physic_world,
        );

        physic_world.add_collider(
            0.0,
            shape,
            body_handle,
            ::na::one(),
            Material::new(0.0, 0.0),
        );
    }
}
