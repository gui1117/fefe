use gilrs::{Button, EventType};
use specs::{Join, World};
use std::f32::EPSILON;
use winit::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

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
    fn winit_event(
        mut self: Box<Self>,
        event: ::winit::Event,
        world: &mut World,
    ) -> Box<GameState> {
        match event {
            ::winit::Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: MouseButton::Left,
                        state,
                        ..
                    },
                ..
            } => {
                for (_, sr) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::SwordRifle>(),
                ).join()
                {
                    sr.attack = state == ElementState::Pressed;
                }
            }
            ::winit::Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::LShift),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                for (_, sr) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::SwordRifle>(),
                ).join()
                {
                    sr.sword_mode = !(state == ElementState::Pressed);
                }
            }
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
                    WindowEvent::CursorMoved {
                        position: (mut x, mut y),
                        ..
                    },
                ..
            } => {
                let size = world.read_resource::<::resource::WindowSize>().0;
                x -= size.0 as f64 / 2.0;
                y -= size.1 as f64 / 2.0;
                y *= -1.0;
                let angle = y.atan2(x) as f32;
                for (_, aim) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::Aim>(),
                ).join()
                {
                    aim.0 = angle;
                }
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
                velocity = velocity.try_normalize(EPSILON).unwrap_or(::na::zero());
                for (_, velocity_control) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::VelocityControl>(),
                ).join()
                {
                    velocity_control.direction = velocity;
                }
            }
            _ => (),
        }
        self
    }

    fn gilrs_event(self: Box<Self>, event: EventType, world: &mut World) -> Box<GameState> {
        match event {
            EventType::ButtonPressed(Button::LeftTrigger2, _) => {
                for (_, sr) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::SwordRifle>(),
                ).join()
                {
                    sr.sword_mode = false;
                }
            }
            EventType::ButtonReleased(Button::LeftTrigger2, _) => {
                for (_, sr) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::SwordRifle>(),
                ).join()
                {
                    sr.sword_mode = true;
                }
            }
            EventType::ButtonPressed(Button::RightTrigger, _) => {
                for (_, sr) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::SwordRifle>(),
                ).join()
                {
                    sr.attack = true;
                }
            }
            EventType::ButtonReleased(Button::RightTrigger, _) => {
                for (_, sr) in (
                    &world.read_storage::<::component::Player>(),
                    &mut world.write_storage::<::component::SwordRifle>(),
                ).join()
                {
                    sr.attack = false;
                }
            }
            _ => (),
        }
        self
    }

    fn gilrs_gamepad_state(
        self: Box<Self>,
        _id: usize,
        gamepad: &::gilrs::Gamepad,
        world: &mut World,
    ) -> Box<GameState> {
        let px = gamepad
            .axis_data(::gilrs::Axis::LeftStickX)
            .map(|e| e.value())
            .unwrap_or(0.0);
        let py = gamepad
            .axis_data(::gilrs::Axis::LeftStickY)
            .map(|e| e.value())
            .unwrap_or(0.0);

        let (px_circle, py_circle) = square_to_circle(px, py);

        for (_, velocity_control) in (
            &world.read_storage::<::component::Player>(),
            &mut world.write_storage::<::component::VelocityControl>(),
        ).join()
        {
            velocity_control.direction = ::na::Vector2::new(px_circle, py_circle);
        }

        let ax = gamepad
            .axis_data(::gilrs::Axis::RightStickX)
            .map(|e| e.value())
            .unwrap_or(0.0);
        let ay = gamepad
            .axis_data(::gilrs::Axis::RightStickY)
            .map(|e| e.value())
            .unwrap_or(0.0);

        let (ax_circle, ay_circle) = square_to_circle(ax, ay);

        for (_, aim) in (
            &world.read_storage::<::component::Player>(),
            &mut world.write_storage::<::component::Aim>(),
        ).join()
        {
            **aim = ay_circle.atan2(ax_circle);
        }

        self
    }

    fn paused(&self) -> bool {
        false
    }
}
