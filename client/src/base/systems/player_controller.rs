use crate::base::game_manager::GameManager;

pub struct PlayerData {
    pub speed: f32,
    pub orientation: na::UnitQuaternion<f64>,
    pub fly: bool,
    pub camera_angles: (f64, f64),
}

impl Default for PlayerData {
    fn default() -> Self {
        Self {
            speed: 10.0,
            orientation: na::UnitQuaternion::identity(),
            fly: true,
            camera_angles: (0.0, 0.0),
        }
    }
}

impl GameManager {
    pub fn run_player(&mut self) {
        use crate::base::game_manager::InputType;
        use winit::event::MouseButton;
        use winit::event::VirtualKeyCode::*;
        let input = &self.input;
        let mut player_data = if let Some(character) = &mut self.character {
            &mut character.player_data
        } else {
            return;
        };

        let (mut run, mut jump, mut sit, mut pickup) = (false, false, false, false);

        if input.key_pressed(&InputType::KeyboardButton(LControl)) {
            sit = true;
        }
        if input.key_pressed(&InputType::KeyboardButton(LShift)) {
            run = true;
        }
        if input.key_pressed(&InputType::KeyboardButton(Space)) {
            jump = true;
        }

        let mut movement_direction = na::Vector2::repeat(0.0f32);
        if input.key_pressed(&InputType::KeyboardButton(W)) {
            movement_direction += na::Vector2::new(0.0, 1.0);
        }

        if input.key_pressed(&InputType::KeyboardButton(A)) {
            movement_direction += na::Vector2::new(-1.0, 0.0);
        }

        if input.key_pressed(&InputType::KeyboardButton(S)) {
            movement_direction += na::Vector2::new(0.0, -1.0);
        }

        if input.key_pressed(&InputType::KeyboardButton(D)) {
            movement_direction += na::Vector2::new(1.0, 0.0);
        }

        // Normalize and convert to I8
        let movement_direction_normalized = movement_direction.normalize() * 127.0;

        if input.key_pressed(&InputType::Mouse(winit::event::MouseButton::Left)) {
            player_data.camera_angles.0 += input.mouse_releative.0 as f64 / 100.0;
            player_data.camera_angles.1 += input.mouse_releative.1 as f64 / 100.0;
            player_data.orientation = na::UnitQuaternion::from_euler_angles(
                player_data.camera_angles.1,
                0.0,
                player_data.camera_angles.0,
            );
        }

        let mut prop_spawn = None;
        if let Some(state) = self
            .input
            .was_pressed(&InputType::Mouse(MouseButton::Right))
        {
            if state == &true {
                player_data.fly = !player_data.fly;
            }
        }
        if let Some(state) = input.was_pressed(&InputType::KeyboardButton(E)) {
            pickup = !state;
        }

        if let Some(state) = input.was_pressed(&InputType::KeyboardButton(Key1)) {
            if state == &false {
                prop_spawn = Some(0);
            }
        }
        if let Some(state) = input.was_pressed(&InputType::KeyboardButton(Key2)) {
            if state == &false {
                prop_spawn = Some(1);
            }
        }
        let movement_angle =
            na::UnitQuaternion::from_euler_angles(0.0, 0.0, -player_data.camera_angles.0 as f32);
        let movement_direction_vec3 = na::Vector3::new(
            movement_direction_normalized.x,
            movement_direction_normalized.y,
            0.0,
        );
        let movement_direction = movement_angle.transform_vector(&movement_direction_vec3);

        // Update state
        self.state = shared::commands::ClientCommand {
            movement_direction: na::Vector2::new(
                movement_direction.x as i8,
                movement_direction.y as i8,
            ),
            orientation: player_data.orientation, //self.orientation,
            fly: player_data.fly,
            run,
            jump,
            sit,
            pickup,
            prop_spawn,
        };
    }
}
