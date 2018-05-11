use nphysics2d::force_generator::ForceGenerator;
use nphysics2d::math::Force;
use nphysics2d::object::{BodyHandle, BodySet};
use nphysics2d::solver::IntegrationParameters;

pub struct StepForces {
    pub forces: Vec<(BodyHandle, Force<f32>)>,
}

impl StepForces {
    pub fn new() -> Self {
        StepForces { forces: vec![] }
    }
}

impl ForceGenerator<f32> for StepForces {
    fn apply(&mut self, _: &IntegrationParameters<f32>, bodies: &mut BodySet<f32>) -> bool {
        for &(body, ref force) in &self.forces {
            if bodies.contains(body) {
                let mut part = bodies.body_part_mut(body);
                part.apply_force(force);
            }
        }

        true
    }
}
