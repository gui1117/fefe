#[macro_use]
extern crate derive_deref;
#[macro_use]
extern crate failure;
extern crate fps_counter;
extern crate gilrs;
#[macro_use]
extern crate lazy_static;
extern crate lyon;
extern crate nalgebra as na;
extern crate nphysics2d as nphysics;
extern crate ncollide;
extern crate png;
extern crate rand;
extern crate ron;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate specs;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate winit;
extern crate alga;
#[macro_use]
extern crate conrod;

pub use nphysics::math as npm;
pub use nphysics::algebra as npa;

mod component;
mod map;
mod entity;
mod animation;
mod system;
mod configuration;
mod resource;
#[macro_use]
mod util;
mod game_state;
mod graphics;

pub use configuration::CFG;

use game_state::GameState;
use vulkano_win::VkSurfaceBuild;
use vulkano::instance::Instance;
use std::time::Duration;
use std::time::Instant;
use std::thread;

fn main() {
    let mut gilrs = gilrs::Gilrs::new();

    let instance = {
        let extensions = vulkano_win::required_extensions();
        let info = app_info_from_cargo_toml!();
        Instance::new(Some(&info), &extensions, None).expect("failed to create Vulkan instance")
    };

    let mut events_loop = winit::EventsLoop::new();
    let window = winit::WindowBuilder::new()
        // .with_fullscreen(winit::get_primary_monitor())
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    try_multiple_time!(
        window.window().set_cursor_state(winit::CursorState::Grab),
        100,
        10
    ).unwrap();
    window.window().set_cursor(winit::MouseCursor::NoneCursor);

    let mut graphics = graphics::Graphics::new(&window);
    let mut ui = conrod::UiBuilder::new([1920.0, 1080.0])
        .build();
    let ids = game_state::Ids::new(ui.widget_id_generator());

    let mut world = specs::World::new();
    world.register::<::component::RigidBody>();
    world.register::<::component::AnimationState>();
    world.register::<::component::Life>();
    world.register::<::component::Aim>();
    world.register::<::component::Player>();
    world.register::<::component::GravityToPlayers>();
    world.add_resource(::resource::UpdateTime(0.0));
    world.add_resource(::resource::PhysicWorld::new());
    world.add_resource(::resource::AnimationImages(vec![]));
    world.add_resource(::resource::Camera::new(::na::one()));
    world.maintain();

    let mut update_dispatcher = ::specs::DispatcherBuilder::new()
        .add(::system::PhysicSystem, "physic", &[])
        .add(::system::GravitySystem, "gravity", &[])
        .add_barrier() // Draw barrier
        .add(::system::AnimationSystem, "animation", &[])
        .build();

    let frame_duration = Duration::new(0, (1_000_000_000.0 / ::CFG.fps as f32) as u32);
    let mut fps_counter = fps_counter::FPSCounter::new();
    let mut last_frame_instant = Instant::now();
    let mut last_update_instant = Instant::now();

    let mut game_state = Box::new(game_state::Game) as Box<GameState>;

    ::map::load_map("one".into(), &mut world).unwrap();

    'main_loop: loop {
        // Parse events
        let mut evs = vec![];
        events_loop.poll_events(|ev| {
            evs.push(ev);
        });
        for ev in evs {
            match ev {
                winit::Event::WindowEvent {
                    event: winit::WindowEvent::Focused(true),
                    ..
                } => {
                    try_multiple_time!(
                        window.window().set_cursor_state(winit::CursorState::Normal),
                        100,
                        10
                    ).unwrap();
                    try_multiple_time!(
                        window.window().set_cursor_state(winit::CursorState::Grab),
                        100,
                        10
                    ).unwrap();
                }
                winit::Event::WindowEvent {
                    event: ::winit::WindowEvent::Closed,
                    ..
                } => {
                    break 'main_loop;
                }
                _ => (),
            }
            game_state = game_state.winit_event(ev, &mut world, &mut ui);
        }
        while let Some(ev) = gilrs.next_event() {
            gilrs.update(&ev);
            game_state = game_state.gilrs_event(ev.event, &mut world, &mut ui);
        }
        for (id, gamepad) in gilrs.gamepads() {
            game_state = game_state.gilrs_gamepad_state(id, gamepad, &mut world, &mut ui);
        }

        // Quit
        if game_state.quit() {
            break 'main_loop;
        }

        // Update
        let delta_time = last_update_instant.elapsed();
        last_update_instant = Instant::now();
        world.write_resource::<::resource::UpdateTime>().0 = delta_time
            .as_secs()
            .saturating_mul(1_000_000_000)
            .saturating_add(delta_time.subsec_nanos() as u64)
            as f32 / 1_000_000_000.0;

        update_dispatcher.dispatch(&mut world.res);
        game_state = game_state.update_draw_ui(&mut ui.set_widgets(), &ids, &mut world);
        world.add_resource(ui.draw().owned());

        // Draw
        graphics.draw(&mut world, &window);

        // Sleep
        let elapsed = last_frame_instant.elapsed();
        if let Some(to_sleep) = frame_duration.checked_sub(elapsed) {
            thread::sleep(to_sleep);
        }
        last_frame_instant = Instant::now();
        fps_counter.tick();
    }
}
