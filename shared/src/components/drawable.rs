use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Drawable {
    pub model: String,
    pub shader: String,
}
