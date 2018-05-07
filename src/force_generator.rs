use na::Real;

use nphysics2d::solver::IntegrationParameters;
use nphysics2d::force_generator::{ForceGenerator, ConstantAcceleration};
use nphysics2d::object::{BodyHandle, BodySet};
use nphysics2d::math::{Velocity, Vector, Force};

pub trait DerefConstantAcceleration {
    fn force_generator_handle(&self) -> usize;

    fn get<'a>(&self, world: &'a ::resource::PhysicWorld) -> &'a ConstantAcceleration<f32> {
        world.force_generator(self.force_generator_handle()).downcast_ref().unwrap()
    }

    fn get_mut<'a>(&mut self, world: &'a mut ::resource::PhysicWorld) -> &'a mut ConstantAcceleration<f32> {
        world.force_generator_mut(self.force_generator_handle()).downcast_mut().unwrap()
    }
}

/// Force generator adding -velocity*damping
pub struct Damping<N: Real> {
    parts: Vec<BodyHandle>,
    linear_damping: N,
    angular_damping: N,
}

impl<N: Real> Damping<N> {
    pub fn new(linear_damping: N, angular_damping: N) -> Self {
        Damping {
            parts: Vec::new(),
            linear_damping,
            angular_damping,
        }
    }

    /// Add a body part to be affected by this force generator.
    pub fn add_body_part(&mut self, body: BodyHandle) {
        self.parts.push(body)
    }
}

impl<N: Real> ForceGenerator<N> for Damping<N> {
    fn apply(&mut self, _: &IntegrationParameters<N>, bodies: &mut BodySet<N>) -> bool {
        let mut i = 0;

        while i < self.parts.len() {
            let body = self.parts[i];

            if bodies.contains(body) {
                let mut part = bodies.body_part_mut(body);
                let velocity = part.as_ref().velocity();
                println!("{:?}", velocity);
                part.apply_force(&Force {
                    linear: -velocity.linear*self.linear_damping,
                    angular: -velocity.angular*self.angular_damping,
                });
                i += 1;
            } else {
                let _ = self.parts.swap_remove(i);
            }
        }

        true
    }
}

pub trait DerefDamping {
    fn force_generator_handle(&self) -> usize;

    fn get<'a>(&self, world: &'a ::resource::PhysicWorld) -> &'a Damping<f32> {
        world.force_generator(self.force_generator_handle()).downcast_ref().unwrap()
    }

    fn get_mut<'a>(&mut self, world: &'a mut ::resource::PhysicWorld) -> &'a mut Damping<f32> {
        world.force_generator_mut(self.force_generator_handle()).downcast_mut().unwrap()
    }
}

pub struct VariableAcceleration<N: Real> {
    parts: Vec<BodyHandle>,
    acceleration: Velocity<N>,
}

impl<N: Real> VariableAcceleration<N> {
    pub fn new(linear_acc: Vector<N>, angular_acc: N) -> Self {
        VariableAcceleration {
            parts: Vec::new(),
            acceleration: Velocity::new(linear_acc, angular_acc),
        }
    }

    pub fn reset(&mut self, linear_acc: Vector<N>, angular_acc: N) {
        self.acceleration = Velocity::new(linear_acc, angular_acc);
    }

    /// Add a body part to be affected by this force generator.
    pub fn add_body_part(&mut self, body: BodyHandle) {
        self.parts.push(body)
    }
}

impl<N: Real> ForceGenerator<N> for VariableAcceleration<N> {
    fn apply(&mut self, _: &IntegrationParameters<N>, bodies: &mut BodySet<N>) -> bool {
        let mut i = 0;

        while i < self.parts.len() {
            let body = self.parts[i];

            if bodies.contains(body) {
                let mut part = bodies.body_part_mut(body);
                let force = part.as_ref().inertia() * self.acceleration;
                part.apply_force(&force);
                i += 1;
            } else {
                let _ = self.parts.swap_remove(i);
            }
        }

        true
    }
}

pub trait DerefVariableAcceleration {
    fn force_generator_handle(&self) -> usize;

    fn get<'a>(&self, world: &'a ::resource::PhysicWorld) -> &'a VariableAcceleration<f32> {
        world.force_generator(self.force_generator_handle()).downcast_ref().unwrap()
    }

    fn get_mut<'a>(&mut self, world: &'a mut ::resource::PhysicWorld) -> &'a mut VariableAcceleration<f32> {
        world.force_generator_mut(self.force_generator_handle()).downcast_mut().unwrap()
    }
}
