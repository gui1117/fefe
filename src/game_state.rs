use specs::{Join, World};
use winit::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use nphysics2d::math::Velocity;

pub trait GameState {
    fn update_draw_ui(self: Box<Self>, world: &mut World) -> Box<GameState>;
    fn winit_event(self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState>;
    fn gilrs_event(self: Box<Self>, event: ::gilrs::EventType, world: &mut World)
        -> Box<GameState>;
    fn gilrs_gamepad_state(
        self: Box<Self>,
        id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
    ) -> Box<GameState>;
    fn quit(&self) -> bool {
        false
    }
    fn paused(&self) -> bool;
}

#[derive(Default)]
pub struct Game {
    dir_stack: Vec<usize>,
}

fn square_to_circle(x: f32, y: f32) -> (f32, f32) {
    (
        x * (1.0 - y * y / 2.0).sqrt(),
        y * (1.0 - x * x / 2.0).sqrt(),
    )
}

impl GameState for Game {
    fn update_draw_ui(self: Box<Self>, _world: &mut World) -> Box<GameState> {
        self
    }
    fn winit_event(mut self: Box<Self>, event: ::winit::Event, world: &mut World) -> Box<GameState> {
        match event {
            ::winit::Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::R),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                ::map::load_map("one".into(), world).unwrap();
            }
            ::winit::Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                let dir = match key {
                    VirtualKeyCode::Z => Some(0),
                    VirtualKeyCode::S => Some(1),
                    VirtualKeyCode::Q => Some(2),
                    VirtualKeyCode::D => Some(3),
                    _ => None,
                };
                if let Some(dir) = dir {
                    self.dir_stack.retain(|&d| d != dir);
                    if state == ElementState::Pressed {
                        self.dir_stack.push(dir);
                    }
                }
                let mut velocity = ::na::Vector2::new(0.0, 0.0);
                for dir in &self.dir_stack {
                    match dir {
                        0 => velocity[1] = 1.0,
                        1 => velocity[1] = -1.0,
                        2 => velocity[0] = -1.0,
                        3 => velocity[0] = 1.0,
                        _ => unreachable!(),
                    }
                }
                let conf = world.read_resource::<::resource::Conf>();
                for (_, body) in (
                    &world.read::<::component::Player>(),
                    &world.read::<::component::RigidBody>(),
                ).join()
                {
                    body.get_mut(&mut world.write_resource()).set_velocity(Velocity {
                        linear: velocity * conf.player_velocity,
                        angular: 0.0,
                    });
                }
            }
            _ => (),
        }
        self
    }

    fn gilrs_event(
        self: Box<Self>,
        _event: ::gilrs::EventType,
        _world: &mut World,
    ) -> Box<GameState> {
        self
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
    ) -> Box<GameState> {
        let conf = world.read_resource::<::resource::Conf>();
        let px = gamepad
            .axis_data(::gilrs::Axis::LeftStickX)
            .map(|e| e.value())
            .unwrap_or(0.0);
        let py = gamepad
            .axis_data(::gilrs::Axis::LeftStickY)
            .map(|e| e.value())
            .unwrap_or(0.0);

        let (px_circle, py_circle) = square_to_circle(px, py);

        for (_, body) in (
            &world.read::<::component::Player>(),
            &world.read::<::component::RigidBody>(),
        ).join()
        {
            body.get_mut(&mut world.write_resource()).set_velocity(Velocity {
                linear: ::na::Vector2::new(px_circle, py_circle) * conf.player_velocity,
                angular: 0.0,
            });
        }

        let ax = gamepad
            .axis_data(::gilrs::Axis::RightStickX)
            .map(|e| e.value())
            .unwrap_or(0.0);
        let ay = gamepad
            .axis_data(::gilrs::Axis::RightStickY)
            .map(|e| e.value())
            .unwrap_or(0.0);

        for (_, aim) in (
            &world.read::<::component::Player>(),
            &mut world.write::<::component::Aim>(),
        ).join()
        {
            **aim = ay.atan2(ax);
        }

        self
    }

    fn paused(&self) -> bool {
        false
    }
}
