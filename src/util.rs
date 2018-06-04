use rand::distributions::{IndependentSample, Range};
use rand::ThreadRng;
use retained_storage::Retained;
use specs::{World, Entity, Join};
use std::f32::consts::PI;
use ncollide2d::shape::{Ball, ShapeHandle};
use winit::{
    ElementState, Event, MouseButton, MouseScrollDelta, TouchPhase, VirtualKeyCode, WindowEvent,
};

#[allow(unused)]
macro_rules! try_multiple_time {
    ($e:expr) => {{
        let mut error_timer = 0;
        let mut res = $e;
        while res.is_err() {
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
            error_timer += 1;
            if error_timer > 10 {
                break;
            }
            res = $e;
        }
        res
    }};
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClampFunction {
    pub min_value: f32,
    pub max_value: f32,
    pub min_t: f32,
    pub max_t: f32,
}

impl ClampFunction {
    pub fn compute(&self, t: f32) -> f32 {
        debug_assert!(self.min_t < self.max_t);
        if t <= self.min_t {
            self.min_value
        } else if t >= self.max_t {
            self.max_value
        } else {
            (t - self.min_t) / (self.max_t - self.min_t) * (self.max_value - self.min_value)
                + self.min_value
        }
    }
}

pub fn reset_world(world: &mut World) {
    world.maintain();
    world.delete_all();
    world.write::<::component::RigidBody>().retained();

    let ground = world.create_entity().with(::component::Ground).build();
    world.add_resource(::resource::BodiesMap::new(ground));

    let mut physic_world = ::resource::PhysicWorld::new();
    world.add_resource(::resource::StepForces::new(&mut physic_world));
    world.add_resource(physic_world);
}

pub fn safe_maintain(world: &mut World) {
    world.maintain();
    let mut physic_world = world.write_resource::<::resource::PhysicWorld>();
    let retained = world
        .write::<::component::RigidBody>()
        .retained()
        .iter()
        .map(|r| r.0)
        .collect::<Vec<_>>();
    physic_world.remove_bodies(&retained);
}

pub fn check_world(world: &World) {
    // TOCHECK: all spawner must be checked
    let insertables_map = world.read_resource::<::resource::InsertablesMap>();
    for spawn in world.read::<::component::UniqueSpawner>().join().map(|s| &s.spawn)
        .chain(world.read::<::component::ChamanSpawner>().join().map(|s| &s.spawn))
        .chain(world.read::<::component::TurretPartSpawner>().join().map(|s| &s.spawn))
    {
        assert!(insertables_map.contains_key(spawn))
    }
}

#[inline]
pub fn move_toward(isometry: &mut ::na::Isometry2<f32>, angle: f32, distance: f32) {
    isometry.translation.vector += ::na::Vector2::new(angle.cos(), angle.sin()) * distance;
}

#[inline]
pub fn move_forward(isometry: &mut ::na::Isometry2<f32>, distance: f32) {
    let angle = isometry.rotation.angle();
    move_toward(isometry, angle, distance);
}

pub fn send_event_to_imgui(
    event: &::winit::Event,
    imgui: &mut ::imgui::ImGui,
    mouse_down: &mut [bool; 5],
) {
    match event {
        Event::WindowEvent {
            event: WindowEvent::MouseInput { button, state, .. },
            ..
        } => {
            match button {
                MouseButton::Left => mouse_down[0] = *state == ElementState::Pressed,
                MouseButton::Right => mouse_down[1] = *state == ElementState::Pressed,
                MouseButton::Middle => mouse_down[2] = *state == ElementState::Pressed,
                MouseButton::Other(0) => mouse_down[3] = *state == ElementState::Pressed,
                MouseButton::Other(1) => mouse_down[4] = *state == ElementState::Pressed,
                MouseButton::Other(_) => (),
            }
            imgui.set_mouse_down(&mouse_down);
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved {
                position: (x, y), ..
            },
            ..
        } => imgui.set_mouse_pos(*x as f32, *y as f32),
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            let pressed = input.state == ElementState::Pressed;
            match input.virtual_keycode {
                Some(VirtualKeyCode::Tab) => imgui.set_key(0, pressed),
                Some(VirtualKeyCode::Left) => imgui.set_key(1, pressed),
                Some(VirtualKeyCode::Right) => imgui.set_key(2, pressed),
                Some(VirtualKeyCode::Up) => imgui.set_key(3, pressed),
                Some(VirtualKeyCode::Down) => imgui.set_key(4, pressed),
                Some(VirtualKeyCode::PageUp) => imgui.set_key(5, pressed),
                Some(VirtualKeyCode::PageDown) => imgui.set_key(6, pressed),
                Some(VirtualKeyCode::Home) => imgui.set_key(7, pressed),
                Some(VirtualKeyCode::End) => imgui.set_key(8, pressed),
                Some(VirtualKeyCode::Delete) => imgui.set_key(9, pressed),
                Some(VirtualKeyCode::Back) => imgui.set_key(10, pressed),
                Some(VirtualKeyCode::Return) => imgui.set_key(11, pressed),
                Some(VirtualKeyCode::Escape) => imgui.set_key(12, pressed),
                Some(VirtualKeyCode::A) => imgui.set_key(13, pressed),
                Some(VirtualKeyCode::C) => imgui.set_key(14, pressed),
                Some(VirtualKeyCode::V) => imgui.set_key(15, pressed),
                Some(VirtualKeyCode::X) => imgui.set_key(16, pressed),
                Some(VirtualKeyCode::Y) => imgui.set_key(17, pressed),
                Some(VirtualKeyCode::Z) => imgui.set_key(18, pressed),
                Some(VirtualKeyCode::LControl) | Some(VirtualKeyCode::RControl) => {
                    imgui.set_key_ctrl(pressed)
                }
                Some(VirtualKeyCode::LShift) | Some(VirtualKeyCode::RShift) => {
                    imgui.set_key_shift(pressed)
                }
                Some(VirtualKeyCode::LAlt) | Some(VirtualKeyCode::RAlt) => {
                    imgui.set_key_alt(pressed)
                }
                Some(VirtualKeyCode::LWin) | Some(VirtualKeyCode::RWin) => {
                    imgui.set_key_super(pressed)
                }
                _ => (),
            }
        }
        Event::WindowEvent {
            event:
                WindowEvent::MouseWheel {
                    delta,
                    phase: TouchPhase::Moved,
                    ..
                },
            ..
        } => match delta {
            MouseScrollDelta::LineDelta(_, y) => imgui.set_mouse_wheel(*y),
            MouseScrollDelta::PixelDelta(_, y) => imgui.set_mouse_wheel(*y),
        },
        Event::WindowEvent {
            event: WindowEvent::ReceivedCharacter(c),
            ..
        } => {
            imgui.add_input_character(*c);
        }
        _ => (),
    }
}

pub fn init_imgui() -> ::imgui::ImGui {
    let mut imgui = ::imgui::ImGui::init();
    imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    imgui.set_mouse_draw_cursor(true);
    imgui.set_imgui_key(::imgui::ImGuiKey::Tab, 0);
    imgui.set_imgui_key(::imgui::ImGuiKey::LeftArrow, 1);
    imgui.set_imgui_key(::imgui::ImGuiKey::RightArrow, 2);
    imgui.set_imgui_key(::imgui::ImGuiKey::UpArrow, 3);
    imgui.set_imgui_key(::imgui::ImGuiKey::DownArrow, 4);
    imgui.set_imgui_key(::imgui::ImGuiKey::PageUp, 5);
    imgui.set_imgui_key(::imgui::ImGuiKey::PageDown, 6);
    imgui.set_imgui_key(::imgui::ImGuiKey::Home, 7);
    imgui.set_imgui_key(::imgui::ImGuiKey::End, 8);
    imgui.set_imgui_key(::imgui::ImGuiKey::Delete, 9);
    imgui.set_imgui_key(::imgui::ImGuiKey::Backspace, 10);
    imgui.set_imgui_key(::imgui::ImGuiKey::Enter, 11);
    imgui.set_imgui_key(::imgui::ImGuiKey::Escape, 12);
    imgui.set_imgui_key(::imgui::ImGuiKey::A, 13);
    imgui.set_imgui_key(::imgui::ImGuiKey::C, 14);
    imgui.set_imgui_key(::imgui::ImGuiKey::V, 15);
    imgui.set_imgui_key(::imgui::ImGuiKey::X, 16);
    imgui.set_imgui_key(::imgui::ImGuiKey::Y, 17);
    imgui.set_imgui_key(::imgui::ImGuiKey::Z, 18);
    imgui
}

#[allow(unused)]
pub fn force_damping(
    mass: f32,
    time_to_reach_percent_velocity: f32,
    percent: f32,
    velocity: f32,
) -> (f32, f32) {
    let damping = mass / time_to_reach_percent_velocity * (1.0 - percent).ln();
    let force = damping * velocity;
    (force, damping)
}

pub fn random_normalized(rng: &mut ThreadRng) -> ::na::Vector2<f32> {
    let angle = Range::new(0.0, 2.0 * PI).ind_sample(rng);
    ::na::Vector2::new(angle.cos(), angle.sin())
}

pub fn vector_zero() -> ::na::Vector2<f32> {
    ::na::zero()
}

pub fn default_shape_handle() -> ShapeHandle<f32> {
    ShapeHandle::new(Ball::new(0.1))
}

pub fn uninitialized_entity() -> Entity {
    unsafe {
        ::std::mem::uninitialized()
    }
}
