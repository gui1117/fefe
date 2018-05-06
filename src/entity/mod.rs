use lyon::tessellation::{FillTessellator, VertexBuffers, FillOptions, FillVertex};
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::svg::path::default::Path;
use lyon::svg::path::PathEvent;
use nphysics2d::object::BodyStatus;
use ncollide2d::shape::{ShapeHandle, Ball, Triangle};
use specs::World;

#[derive(Deref, DerefMut)]
pub struct FillPosition(Vec<[::na::Point2<f32>; 3]>);

impl ::map::TryFromPath for FillPosition {
    fn try_from_path(value: Path) -> Result<Self, ::failure::Error> {
        let mut buffers: VertexBuffers<FillVertex> = VertexBuffers::new();

        {
            let mut vertex_builder = simple_builder(&mut buffers);
            let mut tessellator = FillTessellator::new();

            tessellator.tessellate_path(
                value.path_iter(),
                &FillOptions::default().with_normals(false),
                &mut vertex_builder,
            )
                .map_err(|e| format_err!("invalid path for FillPosition: {:?} for path: \"{:?}\"", e, value))?;
        }

        let mut indices_iter = buffers.indices.iter();
        let mut position = vec![];
        while let (Some(i1), Some(i2), Some(i3)) = (indices_iter.next(), indices_iter.next(), indices_iter.next()) {
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
    Insertable, insert, InsertPosition, InsertableObject {
        Player,
    }
);

object!(
    Fillable, fill, FillPosition, FillableObject {
        Wall,
    }
);

#[derive(Serialize, Deserialize)]
pub struct Wall;

impl Fillable for Wall {
    fn fill(&self, position: FillPosition, world: &mut World) {
        for position in position.iter() {
            unimplemented!();
            // let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

            // let body_handle = physic_world.add_rigid_body(::na::Isometry2::new(::na::Vector2::new(0.0, 0.0), 0.0), ::npm::Inertia::zero());
            // physic_world.rigid_body_mut(body_handle).unwrap().set_status(BodyStatus::Static);

            // physic_world.add_collider(
            //     0.0,
            //     ShapeHandle::new(Ball::new(::CFG.player_radius).clone()),
            //     body_handle,
            //     ::na::one(),
            // );


            // // let body_handle = physic_world.add_rigid_body(::na::one(), ::npm::Inertia::zero());
            // // physic_world.rigid_body_mut(body_handle).unwrap().set_status(BodyStatus::Static);

            // // physic_world.add_collider(
            // //     0.0,
            // //     ShapeHandle::new(Triangle::from_array(&position).clone()),
            // //     body_handle,
            // //     ::na::one(),
            // // );
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Player;

impl Insertable for Player {
    fn insert(&self, position: InsertPosition, world: &mut World) {
        unimplemented!();
        // let p = world
        //     .create_entity()
        //     .with(::component::AnimationState::new(
        //         ::animation::AnimationSpecie::Character,
        //         ::animation::AnimationName::IdleRifle,
        //     ))
        //     .with(::component::Player)
        //     .with(::component::Aim(position.rotation.angle()))
        //     .with(::component::Life(1))
        //     .build();

        // let body_handle = ::component::RigidBody::safe_insert(
        //     p, position.0, ::npm::Inertia::zero(),
        //     BodyStatus::Dynamic,
        //     &mut world.write(),
        //     &mut world.write_resource(),
        // );
        // world.write_resource::<::resource::PhysicWorld>()
        //     .add_collider(
        //         0.0,
        //         ShapeHandle::new(Ball::new(::CFG.player_radius)),
        //         body_handle,
        //         ::na::one(),
        //     );
    }
}
