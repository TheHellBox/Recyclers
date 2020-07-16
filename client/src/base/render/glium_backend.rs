use crate::base::components::Camera;
use crate::base::game_manager::GameManager;
use crate::base::render::backend::BackEnd;
use crate::base::render::shaders::compile_shaders;
use glium::texture::SrgbTexture2d;
use glium::vertex::VertexBufferAny;
use glium::IndexBuffer;
use shared::components::drawable::Drawable;
use shared::components::Transform;

use glium::backend::Facade;
use glium::{glutin, implement_vertex, uniform, Surface};
use std::collections::HashMap;

use renderdoc::{RenderDoc, V110};

pub struct ModelNode {
    pub vertex_buffer: VertexBufferAny,
    pub index_buffer: IndexBuffer<u32>,
    pub texture: String,
    pub transform: na::Matrix4<f32>,
    pub verticies_count: usize,
}

pub struct Model(pub Vec<ModelNode>);

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub normal: [f32; 3],
}
implement_vertex!(Vertex, position, uv, normal);

pub struct GliumBackend {
    pub display: glium::Display,
    shaders: HashMap<String, glium::Program>,
    models: HashMap<String, Model>,
    textures: HashMap<String, SrgbTexture2d>,
    renderdoc: Option<RenderDoc<V110>>,
    resolution: (u32, u32),
}

impl GliumBackend {
    pub fn new(
        window_builder: winit::window::WindowBuilder,
        event_loop: &winit::event_loop::EventLoop<()>,
        resolution: (u32, u32),
    ) -> Self {
        let context = glutin::ContextBuilder::new()
            .with_depth_buffer(24)
            .with_vsync(false)
            .with_srgb(true);
        let display = glium::Display::new(window_builder, context, event_loop).unwrap();
        let shaders = compile_shaders(&display);
        // TODO: Use command line argument instead of renderdoc detection
        // And tbh, renderdoc has to be optional(Use feature)
        let renderdoc = {
            let rd = RenderDoc::new();
            if let Ok(rd) = rd {
                println!("[Client] RenderDoc is enabled!");
                Some(rd)
            } else {
                None
            }
        };
        Self {
            display,
            shaders,
            models: HashMap::with_capacity(1024),
            textures: HashMap::with_capacity(1024),
            renderdoc,
            resolution,
        }
    }

    pub fn draw_skybox<S: glium::Surface + ?Sized>(
        &mut self,
        target: &mut S,
        camera_position: na::Vector3<f64>,
        planet_radius: f64,
    ) {
        let alt = na::distance(
            &na::Point3::from(camera_position),
            &na::Point3::new(0.0, 0.0, 0.0),
        ) - planet_radius;

        let atmo_density = 1.0 - (alt / 160000.0) as f32;
        // That's very stupid lol
        target.clear_color_and_depth(
            (
                0.52 * atmo_density,
                0.8 * atmo_density,
                0.92 * atmo_density,
                1.0,
            ),
            0.0,
        );
    }
    pub fn draw_drawable<S: glium::Surface + ?Sized>(
        &mut self,
        target: &mut S,
        drawable: &Drawable,
        transform: &Transform,
        projection: na::Matrix4<f32>,
        view: na::Matrix4<f64>,
    ) {
        let transform: [[f32; 4]; 4] =
            na::convert::<_, na::Matrix4<f32>>(view * transform.transform_matrix()).into();
        let projection: [[f32; 4]; 4] = projection.into();
        let view: [[f32; 4]; 4] = na::convert::<_, na::Matrix4<f32>>(view).into();
        if !self.models.contains_key(&drawable.model) {
            let model = crate::base::gltf_loader::load_model(
                std::path::Path::new(&drawable.model),
                &self.display,
            );
            self.models.insert(drawable.model.to_string(), model);
        }
        let model = &self.models[&drawable.model];
        for node in &model.0 {
            if !self.textures.contains_key(&node.texture) {
                self.textures.insert(
                    node.texture.clone(),
                    crate::base::textures::load_texture(
                        std::path::Path::new(&node.texture),
                        &self.display,
                    ),
                );
            }
            let texture = &self.textures[&node.texture];
            let node_transform: [[f32; 4]; 4] = node.transform.into();
            target
                .draw(
                    &node.vertex_buffer,
                    &node.index_buffer,
                    &self.shaders[&drawable.shader],
                    &uniform!(
                        projection: projection,
                        view: view,
                        transform: transform,
                        node_transform: node_transform,
                        tex: texture
                    ),
                    &glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::DepthTest::IfMore,
                            write: true,
                            range: (0.0, 1.0),
                            ..Default::default()
                        },
                        backface_culling:
                            glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                        blend: glium::draw_parameters::Blend::alpha_blending(),
                        //polygon_mode: glium::draw_parameters::PolygonMode::Line,
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }
}

impl BackEnd for GliumBackend {
    fn render(&mut self, game_manager: &mut GameManager) {
        if let Some(rd) = &mut self.renderdoc {
            rd.trigger_capture();
        };

        let (x, y) = self.resolution;

        let mut window_frame =
            glium::Frame::new(self.display.get_context().clone(), self.resolution);
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let surface_texture = glium::texture::SrgbTexture2d::empty_with_format(
            &self.display,
            glium::texture::SrgbFormat::U8U8U8,
            glium::texture::MipmapsOption::NoMipmap,
            x,
            y,
        )
        .unwrap();
        let depth_texture = glium::texture::DepthTexture2d::empty_with_format(
            &self.display,
            glium::texture::DepthFormat::F32,
            glium::texture::MipmapsOption::NoMipmap,
            x,
            y,
        )
        .unwrap();
        let mut frame = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
            &self.display,
            &surface_texture,
            &depth_texture,
        )
        .unwrap();

        for (camera_entity, (camera_transform, camera)) in
            &mut game_manager.world.query::<(&Transform, &Camera)>()
        {
            let camera_transform_matrix = camera_transform.calculate_view();
            let planet = &mut game_manager.planet;
            let alt = na::distance(
                &na::Point3::from(camera_transform.isometry.translation.vector),
                &na::Point3::new(0.0, 0.0, 0.0),
            ) - planet.radius;

            // Change znear depending on distance from the planet
            // Yes, that's stupid, I know
            // Mostly I do this because water starts to glitch when viewed from the orbit
            let projection = {
                if alt < 8000.0 {
                    camera.projection(self.resolution, 0.3).to_homogeneous()
                } else {
                    camera
                        .projection(self.resolution, (alt / 3.0) as f32)
                        .to_homogeneous()
                }
            };

            // Draw planet
            self.draw_skybox(
                &mut frame,
                camera_transform.isometry.translation.vector,
                planet.radius,
            );
            game_manager.planet.draw(
                &self.display,
                &mut frame,
                projection,
                camera_transform_matrix,
                &self.shaders,
                &self.models,
                &self.textures,
                &game_manager.world,
                game_manager.time,
                camera_entity,
            );
            // Draw generic objects
            for (_entity, (drawable, transform)) in
                &mut game_manager.world.query::<(&Drawable, &Transform)>()
            {
                self.draw_drawable(
                    &mut frame,
                    drawable,
                    transform,
                    projection,
                    camera_transform_matrix.into(),
                )
            }
        }

        let quad = quad_mesh(&self.display);
        window_frame
            .draw(
                &quad,
                &index_buffer,
                &self.shaders["POST_PR_SIMPLE"],
                &uniform!(tex: &surface_texture, depth: &depth_texture),
                &glium::DrawParameters {
                    depth: glium::Depth {
                        write: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .unwrap();

        window_frame.finish().unwrap();
    }
    fn request_redraw(&mut self) {
        self.display.gl_window().window().request_redraw();
    }
}

pub fn quad_mesh<F: glium::backend::Facade + ?Sized>(facade: &F) -> VertexBufferAny {
    glium::VertexBuffer::new(
        facade,
        &[
            Vertex {
                position: [-1.0, -1.0, 0.0],
                uv: [0.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                uv: [0.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                uv: [1.0, 0.0],
                normal: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                uv: [1.0, 1.0],
                normal: [0.0, 0.0, 0.0],
            },
        ],
    )
    .unwrap()
    .into()
}
