use crate::base::render::glium_backend::{Model, ModelNode, Vertex};
use glium::backend::Facade;
use glium::vertex::VertexBuffer;

pub fn load_model<F: Facade + ?Sized>(path: &std::path::Path, facade: &F) -> Model {
    use glium::index::PrimitiveType::*;
    use gltf::mesh::Mode;

    let mut result = vec![];
    let (document, buffers, _images) = gltf::import(path).unwrap();

    let directory = path.parent().unwrap_or(std::path::Path::new("./assets/"));

    for node in document.nodes() {
        let mesh = node.mesh();
        if mesh.is_none() {
            continue;
        }
        let mesh = mesh.unwrap();
        let transform = node.transform().matrix();

        for primitive in mesh.primitives() {
            let mut texture_path = String::from("./assets/textures/grass.png");
            let mode = match primitive.mode() {
                Mode::Points => Points,
                Mode::Lines => LinesList,
                Mode::LineLoop => LineLoop,
                Mode::LineStrip => LineStrip,
                Mode::Triangles => TrianglesList,
                Mode::TriangleStrip => TriangleStrip,
                Mode::TriangleFan => TriangleFan,
            };

            let material = primitive.material().pbr_metallic_roughness();
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let verticies = reader.read_positions();
            let normals = reader.read_normals();
            let tex_coords = reader.read_tex_coords(0);
            let indices = reader.read_indices();

            if verticies.is_none() || normals.is_none() || indices.is_none() {
                println!("Failed to load primitive, skipping...",);
                println!("verticies: {}", verticies.is_some());
                println!("normals: {}", normals.is_some());
                println!("tex_coords: {}", tex_coords.is_some());
                println!("indices: {}", indices.is_some());

                continue;
            }
            if let Some(info) = material.base_color_texture() {
                let texture = info.texture().source().source();
                if let gltf::image::Source::Uri { uri, mime_type: _ } = texture {
                    texture_path = directory.join(uri).to_str().unwrap().to_string();
                }
            }

            let verticies = verticies.unwrap();
            let normals = normals.unwrap();
            //let tex_coords = tex_coords.unwrap().into_f32();
            let indices: Vec<u32> = indices.unwrap().into_u32().collect();

            let mut verticies: Vec<Vertex> = verticies
                .zip(normals)
                .map(|(vertex, normal)| Vertex {
                    position: [vertex[2], vertex[1], vertex[0]],
                    normal,
                    uv: [0.0, 0.0],
                })
                .collect();

            if let Some(tex_coords) = tex_coords {
                for (i, uv) in tex_coords.into_f32().enumerate() {
                    verticies[i].uv = uv;
                }
            }

            let index_buffer = glium::index::IndexBuffer::new(facade, mode, &indices).unwrap();
            let vertex_buffer = VertexBuffer::new(facade, &verticies).unwrap().into();
            result.push(ModelNode {
                vertex_buffer,
                index_buffer,
                texture: texture_path,
                transform: transform.into(),
                verticies_count: verticies.len(),
            })
        }
    }
    Model(result)
}
