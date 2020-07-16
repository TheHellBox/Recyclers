extern crate nalgebra as na;
pub mod base;

use base::game_manager::GameManager;
use base::render::backend::BackEnd;
use base::render::build_glutin_window;
use base::systems;

use glium::glutin;
use hecs::World;

fn main() {
    std::thread::spawn(move || {
        server::run();
    });

    let netclient = base::network::spawn();
    let world = World::new();

    let mut player_controller = systems::PlayerController::default();

    let planet_radius = 1275620.0;

    let (window_builder, event_loop) = build_glutin_window(1920., 1080., "Silicon Postlive");

    let mut glium_backend =
        base::render::glium_backend::GliumBackend::new(window_builder, &event_loop, (1920, 1080));

    let terrain_textures = base::textures::texture_array(
        vec![
            std::path::Path::new("./assets/textures/grass.png"),
            std::path::Path::new("./assets/textures/sand.png"),
            std::path::Path::new("./assets/textures/rock.png"),
        ],
        &glium_backend.display,
    );
    let planet = base::planet::Planet::new(planet_radius, terrain_textures);

    // That's kinda ugly
    let mut game_manager = GameManager {
        window_events: vec![],
        input: Default::default(),
        planet: planet,
        netclient,
        world,
        time: 0.0,
        entity_ids: std::collections::HashMap::new(),
        // TODO: Implement default
        state: shared::commands::ClientCommand {
            movement_direction: na::Vector2::repeat(127),
            orientation: na::UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            fly: true,
            run: false,
            jump: false,
            sit: false,
            pickup: false,
            prop_spawn: None,
        },
        server_info: None,
        character: None,
    };

    let start = std::time::Instant::now();

    event_loop.run(move |event, _, _control_flow| {
        match event {
            glutin::event::Event::WindowEvent { event, .. } => {
                game_manager.window_events.push(event.to_static().unwrap());
            }
            glutin::event::Event::RedrawRequested(_) => {
                game_manager.time = start.elapsed().as_secs_f32();

                game_manager.run();
                // Move player_controller to game_manager
                player_controller.run(&mut game_manager);
                glium_backend.render(&mut game_manager);
                game_manager.window_events.clear();
                glium_backend.request_redraw();
            }
            _ => {}
        };
    });
}
