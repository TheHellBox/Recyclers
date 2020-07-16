use crate::base::game_manager::GameManager;

pub trait BackEnd {
    fn render(&mut self, _game_manager: &mut GameManager);
    fn request_redraw(&mut self);
}
