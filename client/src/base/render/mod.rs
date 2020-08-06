pub mod ash_backend;
pub mod backend;
pub mod glium_backend;
pub mod planet;
pub mod shaders;

use glium::glutin;

pub fn build_glutin_window(
    x: f32,
    y: f32,
    title: &'static str,
) -> (
    winit::window::WindowBuilder,
    winit::event_loop::EventLoop<()>,
) {
    let window_builder = winit::window::WindowBuilder::new()
        .with_inner_size(glutin::dpi::LogicalSize::new(x, y))
        .with_title(title);
    let event_loop = glutin::event_loop::EventLoop::new();
    (window_builder, event_loop)
}
