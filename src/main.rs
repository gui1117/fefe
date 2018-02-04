extern crate specs;
extern crate rand;
extern crate ron;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate failure;
extern crate lyon;
extern crate serde;
#[macro_use]
extern crate lazy_static;
extern crate gilrs;
extern crate winit;
#[macro_use]
extern crate vulkano;
#[macro_use]
extern crate vulkano_shader_derive;
extern crate vulkano_win;
extern crate fps_counter;
extern crate png;

mod component;
mod map;
mod entity;
mod animation;
mod configuration;
mod resource;
#[macro_use]
mod util;
mod game_state;
mod graphics;

pub use configuration::CFG;

use game_state::GameState;
use vulkano_win::VkSurfaceBuild;
use vulkano::swapchain;
use vulkano::sync::now;
use vulkano::sync::GpuFuture;
use vulkano::instance::Instance;
use winit::{DeviceEvent, Event, WindowEvent};
use std::sync::Arc;
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

    try_multiple_time!(window.window().set_cursor_state(winit::CursorState::Grab), 100, 10).unwrap();

    let mut graphics = graphics::Graphics::new(&window);

    let mut world = specs::World::new();
    world.add_resource(::resource::Drawer::new());
    world.maintain();

    let frame_duration = Duration::new(
        0,
        (1_000_000_000.0 / ::CFG.fps as f32) as u32,
    );
    let mut fps_counter = fps_counter::FPSCounter::new();
    let mut last_frame_instant = Instant::now();
    let mut last_update_instant = Instant::now();

    let mut game_state = Box::new(game_state::Menu) as Box<GameState>;

    loop {
        // Parse events
        let mut evs = vec![];
        events_loop.poll_events(|ev| {
            // TODO regrab on focus(on) ?
            evs.push(ev);
        });
        for ev in evs {
            game_state = game_state.winit_event(ev, &mut world);
        }
        while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
            game_state = game_state.gilrs_event(event, &mut world);
        }

        // Quit
        if game_state.quit() {
            break;
        }

        // Update
        let delta_time = last_update_instant.elapsed();
        last_update_instant = Instant::now();

        // UPDATE WORLD delta_time
        // with delta modified by gamestate (0.0 for pause)
        // or maybe different dispatch depending on gamestate::state {
        // Pause,
        // Play,
        // }

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
