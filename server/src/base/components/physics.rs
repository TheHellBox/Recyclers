use nphysics3d::object::{DefaultBodyHandle, DefaultColliderHandle};

#[derive(Clone)]
pub struct PhysicsBody {
    pub handle: DefaultBodyHandle,
    pub collides_with: Vec<(DefaultColliderHandle, DefaultBodyHandle)>,
    pub on_surface: bool,
}

impl PhysicsBody {
    pub fn new(handle: DefaultBodyHandle) -> Self {
        Self {
            on_surface: false,
            collides_with: vec![],
            handle,
        }
    }
}
