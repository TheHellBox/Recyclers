pub fn load_model(
    path: &std::path::Path,
) -> (
    Vec<na::Point3<f64>>,
    Vec<na::Vector3<f64>>,
    Vec<na::Point3<u32>>,
) {
    let mut result_verticies = vec![];
    let mut result_normals = vec![];
    let mut result_indices = vec![];

    let (document, buffers, _images) = gltf::import(path).unwrap();
    for node in document.nodes() {
        let mesh = node.mesh();
        if mesh.is_none() {
            continue;
        }
        let mesh = mesh.unwrap();
        let transform: na::Matrix4<f32> = node.transform().matrix().into();
        let transform: na::Matrix4<f64> = na::convert(transform);

        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
            let verticies = reader.read_positions();
            let normals = reader.read_normals();
            let indices = reader.read_indices();

            if verticies.is_none() || normals.is_none() || indices.is_none() {
                println!("Failed to load primitive, skipping...",);
                println!("verticies: {}", verticies.is_some());
                println!("normals: {}", normals.is_some());
                println!("indices: {}", indices.is_some());
                continue;
            }
            let verticies = verticies.unwrap();
            let normals = normals.unwrap();
            let mut indices: Vec<na::Point3<u32>> = indices
                .unwrap()
                .into_u32()
                .collect::<Vec<u32>>()
                .chunks(3)
                .map(|i| na::Point3::new(i[0], i[1], i[2]))
                .collect();
            let mut verticies: Vec<na::Point3<f64>> = verticies
                .map(|vertex| {
                    transform.transform_point(&na::Point3::new(
                        vertex[0] as f64,
                        vertex[1] as f64,
                        vertex[2] as f64,
                    ))
                })
                .collect();
            let mut normals: Vec<na::Vector3<f64>> = normals
                .map(|normal| {
                    na::Vector3::new(normal[0] as f64, normal[1] as f64, normal[2] as f64)
                })
                .collect();
            result_verticies.append(&mut verticies);
            result_normals.append(&mut normals);
            result_indices.append(&mut indices);
        }
    }
    (result_verticies, result_normals, result_indices)
}

pub fn load_convex(path: &std::path::Path) -> ncollide3d::shape::ConvexHull<f64> {
    let (verticies, _normals, indices) = load_model(path);
    ncollide3d::shape::ConvexHull::try_from_points(
        &verticies,
        //indices
        //    .iter()
        //    .map(|x| na::Point3::new(x[0] as usize, x[1] as usize, x[2] as usize))
        //    .collect(),
        //None,
    )
    .unwrap()
}
