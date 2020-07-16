use glium::backend::Facade;
use glium::texture::srgb_texture2d::SrgbTexture2d;

pub fn load_raw(
    path: &std::path::Path,
) -> std::io::Result<(glium::texture::RawImage2d<u8>, png::OutputInfo)> {
    use png::ColorType::*;
    let decoder = png::Decoder::new(std::fs::File::open(path)?);
    let (info, mut reader) = decoder.read_info()?;
    let mut img_data = vec![0; info.buffer_size()];
    reader.next_frame(&mut img_data)?;

    match info.color_type {
        RGB => Ok((
            glium::texture::RawImage2d::from_raw_rgb(img_data, (info.width, info.height)),
            info,
        )),
        RGBA => Ok((
            glium::texture::RawImage2d::from_raw_rgba(img_data, (info.width, info.height)),
            info,
        )),
        _ => unreachable!(
            "Error: Unrecognized image format. Please use RGB/RGBA textures({})",
            path.display()
        ),
    }
}

pub fn load_texture<F: Facade + ?Sized>(path: &std::path::Path, facade: &F) -> SrgbTexture2d {
    let (raw, _) = load_raw(path).unwrap();
    SrgbTexture2d::new(facade, raw).unwrap()
}

pub fn texture_array<F: Facade + ?Sized>(
    pathes: Vec<&std::path::Path>,
    facade: &F,
) -> glium::texture::SrgbTexture2dArray {
    let mut textures = vec![];
    for path in pathes {
        textures.push(load_raw(path).unwrap().0);
    }
    let array = glium::texture::SrgbTexture2dArray::new(facade, textures).unwrap();
    array
}
