use entity::{FillPosition, Fillable, Segmentable, SegmentsPosition};
use ncollide2d::shape::{ConvexPolygon, Segment, ShapeHandle};
use nphysics2d::object::{BodyHandle, BodyStatus, Material};
use specs::World;

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
        let mut physic_world = world.write_resource::<::resource::PhysicWorld>();

        for position in position.iter() {
            physic_world.add_collider(
                0.0,
                ShapeHandle::new(Segment::from_array(&position).clone()),
                BodyHandle::ground(),
                ::na::one(),
                Material::new(0.0, 0.0),
            );
        }
    }
}
