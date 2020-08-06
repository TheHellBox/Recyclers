use crate::base::planet::Planet;
use crate::base::systems::player_controller::PlayerData;
use shared::EntityId;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum InputType {
    KeyboardButton(glium::glutin::event::VirtualKeyCode),
    ControllerButton(gilrs::Button, gilrs::GamepadId),
    Mouse(glium::glutin::event::MouseButton),
}

#[derive(Default)]
pub struct Input {
    pub keys_state: HashMap<InputType, bool>,
    pub keys_updated: HashMap<InputType, bool>,
    pub mouse_position: (f32, f32),
    pub mouse_releative: (f32, f32),
}
#[allow(dead_code)]
impl Input {
    pub fn key_pressed(&self, keycode: &InputType) -> bool {
        self.keys_state.get(keycode).unwrap_or(&false).clone()
    }
    // Not sure if naming is any good
    pub fn was_pressed(&self, keycode: &InputType) -> Option<&bool> {
        self.keys_updated.get(keycode)
    }
}

pub struct Character {
    pub entity: hecs::Entity,
    pub camera: hecs::Entity,
    pub player_data: PlayerData,
}

pub struct GameManager {
    //pub event_loop: glium::glutin::event_loop::EventLoop<()>,
    pub window_events: Vec<glium::glutin::event::WindowEvent<'static>>,

    pub planet: Planet,
    pub input: Input,

    pub netclient: crate::base::network::Client,

    pub state: shared::commands::ClientCommand,
    pub server_info: Option<shared::commands::ServerInfo>,

    pub time: f32,
    pub since_input_sent: std::time::Duration,
    pub delta: std::time::Duration,
    pub world: hecs::World,
    pub entity_ids: HashMap<EntityId, hecs::Entity>,
    pub character: Option<Character>,
}

impl GameManager {
    pub fn run(&mut self) {
        use glium::glutin::event::WindowEvent::*;
        self.input.mouse_releative.0 = 0.0;
        self.input.mouse_releative.1 = 0.0;

        self.input.keys_updated.clear();
        // Update input
        for event in &self.window_events {
            match event {
                KeyboardInput { input, .. } => {
                    if let Some(virtual_keycode) = input.virtual_keycode {
                        self.input.keys_state.insert(
                            InputType::KeyboardButton(virtual_keycode),
                            input.state == glium::glutin::event::ElementState::Pressed,
                        );
                        self.input.keys_updated.insert(
                            InputType::KeyboardButton(virtual_keycode),
                            input.state == glium::glutin::event::ElementState::Pressed,
                        );
                    }
                }
                MouseInput { state, button, .. } => {
                    self.input.keys_state.insert(
                        InputType::Mouse(button.clone()),
                        state == &glium::glutin::event::ElementState::Pressed,
                    );
                    self.input.keys_updated.insert(
                        InputType::Mouse(button.clone()),
                        state == &glium::glutin::event::ElementState::Pressed,
                    );
                }
                CursorMoved { position, .. } => {
                    self.input.mouse_releative.0 = position.x as f32 - self.input.mouse_position.0;
                    self.input.mouse_releative.1 = position.y as f32 - self.input.mouse_position.1;

                    self.input.mouse_position.0 = position.x as f32;
                    self.input.mouse_position.1 = position.y as f32;
                }
                _ => {}
            }
        }

        self.since_input_sent += self.delta;
        if let Some(overflow) = self
            .since_input_sent
            .checked_sub(std::time::Duration::from_secs(1) / 60)
        {
            self.run_player();
            self.netclient.network_sender.send(self.state).unwrap();
        }
        // Process server ticks
        while let Ok(command) = self.netclient.network_receiver.try_recv() {
            use crate::base::network::ServerCommand::*;
            match command {
                Tick(tick) => {
                    for (id, components) in tick.spawns {
                        let mut builder = hecs::EntityBuilder::new();
                        self.spawn(&mut builder, id, components);
                    }
                    for (id, isometry) in tick.positions {
                        if let Some(entity) = self.entity_ids.get(&id) {
                            if let Ok(mut transform) =
                                self.world.get_mut::<shared::components::Transform>(*entity)
                            {
                                transform.isometry = isometry;
                            }
                        }
                    }
                }
                ServerInfoUpdate(info) => {
                    println!("[CLIENT] {:?}", info);
                    self.server_info = Some(info);
                }
            }
        }
        shared::components::parent::update_children(&mut self.world);
    }
    pub fn spawn_local_character(&mut self, entity: hecs::Entity) {
        use crate::base::components::*;
        use shared::components::*;
        println!("[CLIENT] Spawn local character!");
        let mut camera = hecs::EntityBuilder::new();
        camera.add(Camera::new());
        camera.add(Parent {
            parent: entity,
            local_transform: Transform::default(),
        });
        camera.add(Transform {
            ..Default::default()
        });
        let camera = self.world.spawn(camera.build());
        self.character = Some(Character {
            entity,
            camera,
            player_data: PlayerData::default(),
        });
    }
    pub fn spawn(
        &mut self,
        builder: &mut hecs::EntityBuilder,
        id: EntityId,
        components: Vec<shared::commands::Component>,
    ) {
        builder.add(id);
        for component in components {
            use shared::commands::Component::*;
            match component {
                Transform(x) => {
                    builder.add(x);
                }
                Drawable(x) => {
                    builder.add(x);
                }
            }
        }
        let e = self.world.spawn(builder.build());
        self.entity_ids.insert(id, e);
        if let Some(server_info) = &self.server_info {
            if server_info.character_id == id.0 {
                self.spawn_local_character(e);
            }
        }
        println!("[CLIENT] Spawn {}", id.0);
    }
}
