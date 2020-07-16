use crate::planet::{Coords, Face};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Chunk {
    pub depth: u8,
    pub coords: Coords,
}

impl Chunk {
    pub fn from_face(face: Face) -> Self {
        Chunk {
            coords: Coords {
                coords: (0, 0),
                face,
            },
            depth: 0,
        }
    }
    pub fn transform(
        &self,
        radius: f64,
        view: na::IsometryMatrix3<f64>,
    ) -> ([f32; 3], [[f32; 4]; 4]) {
        let origin: na::Vector3<f32> = na::convert(radius * self.origin_on_face().into_inner());
        let world = self.coords.face.basis()
            * na::Translation3::from(na::convert::<_, na::Vector3<f64>>(origin));
        let isometry_matrix: na::IsometryMatrix3<f32> =
            na::convert::<_, na::IsometryMatrix3<f32>>(view * world);
        (origin.into(), isometry_matrix.to_homogeneous().into())
    }
    pub fn origin_on_face(&self) -> na::Unit<na::Vector3<f64>> {
        let size = self.edge_length();
        let vec = na::Vector3::new(
            (self.coords.coords.0 as f64 + 0.5) * size - 1.0,
            (self.coords.coords.1 as f64 + 0.5) * size - 1.0,
            1.0,
        );
        na::Unit::new_normalize(vec)
    }
    pub fn resolution(&self) -> u32 {
        2u32.pow(self.depth as u32)
    }
    pub fn edge_length(&self) -> f64 {
        2.0 / self.resolution() as f64
    }
    pub fn samples(&self, chunk_resolution: u32) -> Vec<na::Unit<na::Vector3<f64>>> {
        self.coords.samples(self.resolution(), chunk_resolution)
    }
    pub fn samples_xy(
        &self,
        chunk_resolution: u32,
    ) -> Vec<(na::Unit<na::Vector3<f64>>, (u32, u32))> {
        self.coords.samples_xy(self.resolution(), chunk_resolution)
    }
    pub fn subdivide(&self) -> [Self; 4] {
        let depth = self.depth + 1;
        let face = self.coords.face;
        let (x, y) = (self.coords.coords.0 * 2, self.coords.coords.1 * 2);
        [
            Chunk {
                coords: Coords {
                    coords: (x, y),
                    face,
                },
                depth,
            },
            Chunk {
                coords: Coords {
                    coords: (x, y + 1),
                    face,
                },
                depth,
            },
            Chunk {
                coords: Coords {
                    coords: (x + 1, y),
                    face,
                },
                depth,
            },
            Chunk {
                coords: Coords {
                    coords: (x + 1, y + 1),
                    face,
                },
                depth,
            },
        ]
    }
}
