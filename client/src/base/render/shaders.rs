use std::collections::HashMap;
use std::path::Path;

// I'm really bad at writing shaders. Sorry
pub const VERTEX_POST_PR: &'static str = r#"
    #version 450
    in vec3 position;
    in vec2 uv;
    out vec2 uv_coords;
    void main() {
        uv_coords = uv;
        gl_Position = vec4(position, 1.0);
    }
"#;

pub const POST_PROCESSING_SIMPLE: &'static str = r#"
    #version 450
    in vec2 uv_coords;
    out vec4 color;
    uniform sampler2D tex;
    uniform sampler2D depth;
    void main() {
        color = texture(tex, uv_coords);
    }
"#;

fn load_file(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

pub fn compile_shaders<F: glium::backend::Facade + ?Sized>(
    facade: &F,
) -> HashMap<String, glium::Program> {
    let mut shaders = HashMap::with_capacity(4);

    let planet_shader = glium::Program::from_source(
        facade,
        &load_file(&Path::new("./assets/shaders/planet/planet.vert")),
        &load_file(&Path::new("./assets/shaders/planet/planet.frag")),
        None,
    )
    .unwrap();
    let water_shader = glium::Program::from_source(
        facade,
        &load_file(&Path::new("./assets/shaders/water/water.vert")),
        &load_file(&Path::new("./assets/shaders/water/water.frag")),
        None,
    )
    .unwrap();
    let clouds_shader = glium::Program::from_source(
        facade,
        &load_file(&Path::new("./assets/shaders/clouds/clouds.vert")),
        &load_file(&Path::new("./assets/shaders/clouds/clouds.frag")),
        None,
    )
    .unwrap();
    let simple_shader = glium::Program::from_source(
        facade,
        &load_file(&Path::new("./assets/shaders/simple/simple.vert")),
        &load_file(&Path::new("./assets/shaders/simple/simple.frag")),
        None,
    )
    .unwrap();

    let trees_shader = glium::Program::from_source(
        facade,
        &load_file(&Path::new("./assets/shaders/trees/trees.vert")),
        &load_file(&Path::new("./assets/shaders/trees/trees.frag")),
        None,
    )
    .unwrap();

    let post_pr_simple =
        glium::Program::from_source(facade, VERTEX_POST_PR, POST_PROCESSING_SIMPLE, None).unwrap();
    shaders.insert("PLANET".to_string(), planet_shader);
    shaders.insert("WATER".to_string(), water_shader);
    shaders.insert("CLOUDS".to_string(), clouds_shader);
    shaders.insert("SIMPLE".to_string(), simple_shader);
    shaders.insert("TREES".to_string(), trees_shader);
    shaders.insert("POST_PR_SIMPLE".to_string(), post_pr_simple);

    shaders
}
