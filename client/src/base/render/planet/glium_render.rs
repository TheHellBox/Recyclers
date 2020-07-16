use crate::base::planet::*;
use crate::base::render::glium_backend::Model;
use glium::texture::SrgbTexture2d;
use glium::uniform;
use shared::components::Transform;
use std::collections::HashMap;

#[derive(Copy, Clone)]
struct TreeAttr {
    tree_position: (f32, f32),
    tree_type: u32,
}

glium::implement_vertex!(TreeAttr, tree_position, tree_type);

impl Planet {
    fn draw_water<S: glium::Surface + ?Sized>(
        &self,
        target: &mut S,
        projection: na::Matrix4<f32>,
        shader: &glium::Program,
        time: f32,
        view: na::Isometry3<f64>,
    ) {
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let view_uni: na::Matrix4<f32> = na::convert(view.to_homogeneous());
        let view_uni: [[f32; 4]; 4] = view_uni.into();
        let projection: [[f32; 4]; 4] = projection.into();

        let water_quads = 16usize;
        for water_chunk in &self.water_cache.render {
            let verticies = water_quads.pow(2) * 6;
            let (origin, worldview) = water_chunk.transform(self.radius, na::convert(view));
            target
                .draw(
                    glium::vertex::EmptyVertexAttributes {
                        len: verticies as usize,
                    },
                    &index_buffer,
                    shader,
                    &uniform!(
                        time: time,
                        origin: origin,
                        view: view_uni,
                        transform: worldview,
                        projection: projection,
                        quads: water_quads as i32,
                        radius: self.radius as f32,
                        depth: water_chunk.depth as i32,
                        chunk_coords: [water_chunk.coords.coords.0 as f32, water_chunk.coords.coords.1 as f32],
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

    fn draw_clouds<S: glium::Surface + ?Sized>(
        &self,
        target: &mut S,
        projection: na::Matrix4<f32>,
        shader: &glium::Program,
        time: f32,
        camera_position: na::Vector3<f64>,
        view: na::Isometry3<f64>,
    ) {
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let view_uni: na::Matrix4<f32> = na::convert(view.to_homogeneous());
        let view_uni: [[f32; 4]; 4] = view_uni.into();
        let projection: [[f32; 4]; 4] = projection.into();

        let alt = na::distance(
            &na::Point3::from(camera_position),
            &na::Point3::new(0.0, 0.0, 0.0),
        ) - self.radius;

        // Used to inverse colors
        // Clouds are dark from below and light from above
        let inverse = alt < CLOUDS_HEIGHT;

        // Draw clouds
        let clouds_quads = 8usize;
        for clouds_chunk in &self.clouds_cache.render {
            let verticies = clouds_quads.pow(2) * 6;
            let (origin, worldview) = clouds_chunk.transform(self.radius, na::convert(view));
            target
                .draw(
                    glium::vertex::EmptyVertexAttributes {
                        len: verticies as usize,
                    },
                    &index_buffer,
                    shader,
                    &uniform!(
                        time: time,
                        origin: origin,
                        view: view_uni,
                        transform: worldview,
                        projection: projection,
                        quads: clouds_quads as i32,
                        depth: clouds_chunk.depth as i32,
                        inverse: inverse,
                        radius: (self.radius + CLOUDS_HEIGHT) as f32,
                        chunk_coords: [clouds_chunk.coords.coords.0 as f32, clouds_chunk.coords.coords.1 as f32],
                    ),
                    &glium::DrawParameters {
                        depth: glium::Depth {
                            test: glium::DepthTest::IfMore,
                            write: true,
                            range: (0.0, 1.0),
                            ..Default::default()
                        },
                        blend: glium::draw_parameters::Blend::alpha_blending(),
                        //polygon_mode: glium::draw_parameters::PolygonMode::Line,
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }

    fn draw_surface<S: glium::Surface + ?Sized>(
        &self,
        target: &mut S,
        projection: na::Matrix4<f32>,
        shader: &glium::Program,
        view: na::Isometry3<f64>,
    ) {
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);
        let view_uni: na::Matrix4<f32> = na::convert(view.to_homogeneous());
        let view_uni: [[f32; 4]; 4] = view_uni.into();
        let projection_uni: [[f32; 4]; 4] = projection.into();

        for chunk in &self.surface_cache.render {
            let slot = self.surface_cache.get(chunk);
            if slot.is_none() {
                continue;
            }
            let chunk_data = {
                if let Some(hm) = self.heightmaps.get(&self.surface_cache.get(chunk).unwrap()) {
                    hm
                } else {
                    continue;
                }
            };
            let verticies = CHUNK_QUADS.pow(2) * 6;
            let (origin, worldview) = chunk.transform(self.radius, na::convert(view));
            target
                .draw(
                    glium::vertex::EmptyVertexAttributes {
                        len: verticies as usize,
                    },
                    &index_buffer,
                    &shader,
                    &uniform!(
                        origin: origin,
                        view: view_uni,
                        normals: &chunk_data.normalmap,
                        transform: worldview,
                        heightmap: &chunk_data.heightmap,
                        projection: projection_uni,
                        quads: CHUNK_QUADS as i32,
                        depth: chunk.depth as i32,
                        radius: self.radius as f32,
                        tex: &self.terrain_textures,
                        max_height: i16::MAX as f32 / 12.0,//16000.0f32,
                        chunk_coords: [chunk.coords.coords.0 as f32, chunk.coords.coords.1 as f32],
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
                        //polygon_mode: glium::draw_parameters::PolygonMode::Line,
                        ..Default::default()
                    },
                )
                .unwrap();
        }
    }

    fn draw_trees<S: glium::Surface + ?Sized>(
        &self,
        target: &mut S,
        projection: na::Matrix4<f32>,
        shader: &glium::Program,
        view: na::Isometry3<f64>,
        models: &HashMap<String, Model>,
        textures: &HashMap<String, SrgbTexture2d>,
        display: &glium::Display,
    ) {
        let view_uni: na::Matrix4<f32> = na::convert(view.to_homogeneous());
        let view_uni: [[f32; 4]; 4] = view_uni.into();
        let projection_uni: [[f32; 4]; 4] = projection.into();
        for chunk in &self.surface_cache.render {
            if chunk.depth < 15 {
                continue;
            }
            let slot = self.surface_cache.get(chunk);
            if slot.is_none() {
                continue;
            }
            let chunk_data = {
                if let Some(hm) = self.heightmaps.get(&self.surface_cache.get(chunk).unwrap()) {
                    hm
                } else {
                    continue;
                }
            };
            let nodes = &models["./assets/models/tree/tree.gltf"].0;
            let (origin, worldview) = chunk.transform(self.radius, na::convert(view));
            let origin_p: na::Point3<f32> = origin.into();
            let rotation = na::UnitQuaternion::face_towards(&origin_p.coords, &na::Vector3::z())
                * na::UnitQuaternion::from_euler_angles(3.14 / 2.0, 0.0, 0.0);
            let rotation_matrix: na::Matrix4<f32> = rotation.to_homogeneous();
            // This is the first time I use batching, so my code probably sucks
            let per_instance = {
                let data = chunk_data
                    .trees
                    .iter()
                    .map(|tree| TreeAttr {
                        tree_position: tree.1,
                        tree_type: tree.0 as u32,
                    })
                    .collect::<Vec<_>>();
                glium::vertex::VertexBuffer::dynamic(display, &data).unwrap()
            };
            for node in nodes {
                let texture = &textures[&node.texture];
                let node_transform: [[f32; 4]; 4] = (rotation_matrix * node.transform).into();
                target
                    .draw(
                        (&node.vertex_buffer, per_instance.per_instance().unwrap()),
                        &node.index_buffer,
                        &shader,
                        &uniform!(
                            origin: origin,
                            view: view_uni,
                            transform: worldview,
                            node_transform: node_transform,
                            heightmap: &chunk_data.heightmap,
                            projection: projection_uni,
                            radius: self.radius as f32,
                            max_height: i16::MAX as f32 / 12.0,
                            depth: chunk.depth as i32,
                            tex: texture,
                            chunk_coords: [chunk.coords.coords.0 as f32, chunk.coords.coords.1 as f32],
                        ),
                        &glium::DrawParameters {
                            depth: glium::Depth {
                                test: glium::DepthTest::IfMore,
                                write: true,
                                range: (0.0, 1.0),
                                ..Default::default()
                            },
                            blend: glium::draw_parameters::Blend::alpha_blending(),
                            //backface_culling:
                            //    glium::draw_parameters::BackfaceCullingMode::CullClockwise,
                            ..Default::default()
                        },
                    )
                    .unwrap();
            }
        }
    }

    // Too much arguments. meh.
    // I should probably give the whole glium backend as a argument
    pub fn draw<S: glium::Surface + ?Sized>(
        &mut self,
        display: &glium::Display,
        target: &mut S,
        projection: na::Matrix4<f32>,
        view: na::Isometry3<f64>,
        shaders: &HashMap<String, glium::Program>,
        models: &HashMap<String, Model>,
        textures: &HashMap<String, SrgbTexture2d>,
        world: &hecs::World,
        time: f32,
        camera: hecs::Entity,
    ) {
        let camera_transform = *world.get::<Transform>(camera).unwrap();
        self.update_cache(camera_transform.isometry.translation.vector);
        self.allocate_chunks();

        for (chunk, slot, heightmap, normalmap) in self.output.try_recv() {
            self.surface_cache.release(slot);

            // Generate heightmap
            let heightmap = glium::texture::RawImage2d {
                data: std::borrow::Cow::from(heightmap),
                width: CHUNK_SAMPLES,
                height: CHUNK_SAMPLES,
                format: glium::texture::ClientFormat::I16,
            };
            let heightmap = glium::texture::texture2d::Texture2d::with_format(
                display,
                heightmap,
                glium::texture::UncompressedFloatFormat::I16,
                glium::texture::MipmapsOption::NoMipmap,
            )
            .unwrap();

            // Generate normalmap
            let normals = glium::texture::RawImage2d {
                data: std::borrow::Cow::from(normalmap),
                width: NORMAL_SAMPLES,
                height: NORMAL_SAMPLES,
                format: glium::texture::ClientFormat::I8I8,
            };
            let normalmap = glium::texture::texture2d::Texture2d::with_format(
                display,
                normals,
                glium::texture::UncompressedFloatFormat::I8I8,
                glium::texture::MipmapsOption::NoMipmap,
            )
            .unwrap();

            let trees = self.trees_at(&chunk);
            self.heightmaps.insert(
                slot,
                ChunkData {
                    heightmap,
                    normalmap,
                    trees,
                },
            );
        }

        self.draw_surface(target, projection, &shaders["PLANET"], view);
        self.draw_water(target, projection, &shaders["WATER"], time, view);
        self.draw_clouds(
            target,
            projection,
            &shaders["CLOUDS"],
            time,
            camera_transform.isometry.translation.vector,
            view,
        );
        self.draw_trees(
            target,
            projection,
            &shaders["TREES"],
            view,
            models,
            textures,
            display,
        );
    }
}
