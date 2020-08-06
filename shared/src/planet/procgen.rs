use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub enum PostType {
    Add(f64),
    Sub(f64),
    Mul(f64),
    Div(f64),
    Pow(i32),
    Inv,
    Abs,
}

impl PostType {
    fn apply(&self, value: &mut f64) {
        match self {
            PostType::Add(v) => *value += v,
            PostType::Sub(v) => *value -= v,
            PostType::Mul(v) => *value *= v,
            PostType::Div(v) => *value /= v,
            PostType::Pow(v) => *value = value.powi(*v),
            PostType::Abs => *value = value.abs(),
            PostType::Inv => *value = 1.0 - *value,
        };
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum NoiseType {
    Simplex(i64),
    Fbm {
        lac: f32,
        gain: f32,
        octaves: u8,
        seed: i32,
    },
}

impl NoiseType {
    pub fn get(&self, coords: na::Vector3<f64>) -> f64 {
        //use core::arch::x86_64::_mm_set1_pd;
        //use simdeez::Simd;
        let x = coords.x;
        let y = coords.y;
        let z = coords.z;

        match self {
            Self::Simplex(seed) => unsafe {
                simdnoise::scalar::simplex_3d_f64(x, y, z, *seed) as f64
            },
            Self::Fbm {
                lac,
                gain,
                octaves,
                seed,
            } => unsafe {
                simdnoise::scalar::fbm_3d(
                    x as f32, y as f32, z as f32, *lac, *gain, *octaves, *seed,
                ) as f64
            },
        }
    }
    pub fn get_deriv(&self, coords: na::Vector3<f64>) -> (f64, [f64; 3]) {
        use core::arch::x86_64::_mm256_set1_ps;
        use simdeez::avx2::*;
        use simdeez::Simd;
        let x = coords.x;
        let y = coords.y;
        let z = coords.z;

        match self {
            Self::Simplex(seed) => unsafe {
                (0.0, [0.0, 0.0, 0.0]) //simdnoise::simplex::simplex_3d_deriv(x, y, z, *seed)
            },
            Self::Fbm {
                lac,
                gain,
                octaves,
                seed,
            } => unsafe {
                let x = _mm256_set1_ps(x as f32);
                let y = _mm256_set1_ps(y as f32);
                let z = _mm256_set1_ps(z as f32);
                let lac = _mm256_set1_ps(*lac);
                let gain = _mm256_set1_ps(*gain);

                let simd_result = fbm_3d_deriv::<Avx2>(
                    F32x8(x),
                    F32x8(y),
                    F32x8(z),
                    F32x8(lac),
                    F32x8(gain),
                    *seed,
                    *octaves,
                );
                let mut result = (0.0, [0.0, 0.0, 0.0]);
                Avx2::storeu_ps(&mut result.0, simd_result.0);
                Avx2::storeu_ps(&mut result.1[0], simd_result.1[0]);
                Avx2::storeu_ps(&mut result.1[1], simd_result.1[1]);
                Avx2::storeu_ps(&mut result.1[2], simd_result.1[2]);
                (
                    result.0 as f64,
                    [result.1[0] as f64, result.1[1] as f64, result.1[2] as f64],
                )
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum LayerType {
    Noise { noise: NoiseType, frequency: f64 },
    Value(f64),
}

impl LayerType {
    pub fn get(&self, point: na::Point3<f64>, mask: f64) -> f64 {
        match self {
            LayerType::Noise { noise, frequency } => {
                noise.get((point.coords * *frequency).into()) * mask
            }
            LayerType::Value(v) => *v * mask,
        }
    }
    pub fn get_deriv(&self, point: na::Point3<f64>, mask: f64) -> (f64, [f64; 3]) {
        match self {
            LayerType::Noise { noise, frequency } => {
                noise.get_deriv((point.coords * *frequency).into())
            }
            LayerType::Value(v) => (*v * mask, [0.0, 0.0, 0.0]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Layer {
    pub layer_type: LayerType,
    pub mask: Option<Box<Layer>>,
    pub warp: Option<Box<Layer>>,
    pub post: Vec<PostType>,
    pub depth: f64,
}

impl Layer {
    pub fn get(&self, point: na::Point3<f64>) -> f64 {
        let m = {
            if let Some(mask) = &self.mask {
                mask.get(point)
            } else {
                1.0
            }
        };
        let warp = {
            if let Some(warp_layer) = &self.warp {
                warp_layer.get(point)
            } else {
                0.0
            }
        };
        let mut h = self.layer_type.get(
            na::Point3::from(point.coords + na::Vector3::repeat(warp)),
            m,
        );
        for op in &self.post {
            op.apply(&mut h);
        }
        h *= self.depth;
        h
    }
    pub fn get_deriv(&self, point: na::Point3<f64>) -> (f64, [f64; 3]) {
        let m = {
            if let Some(mask) = &self.mask {
                mask.get(point)
            } else {
                1.0
            }
        };
        let warp = {
            if let Some(warp_layer) = &self.warp {
                warp_layer.get(point)
            } else {
                0.0
            }
        };
        let mut h = self.layer_type.get_deriv(
            na::Point3::from(point.coords + na::Vector3::repeat(warp)),
            m,
        );
        for op in &self.post {
            op.apply(&mut h.0);
        }
        h.0 *= self.depth;
        for i in &mut h.1 {
            *i *= self.depth;
        }
        h
    }
}

pub struct PlanetProcGen {
    pub layers: Vec<Layer>,
    pub file_hash: u64,
    pub file_path: String,
}

impl PlanetProcGen {
    // FIXME: depth is oblsolete
    pub fn get(&self, point: na::Point3<f64>, depth: u8) -> f64 {
        let mut result = 0.0;
        for layer in &self.layers {
            result += layer.get(point);
        }
        result * i16::MAX as f64
    }
    pub fn get_deriv(&self, point: na::Point3<f64>, depth: u8) -> (f64, [f64; 3]) {
        let mut result = (0.0, [0.0, 0.0, 0.0]);
        for layer in &self.layers {
            let noise = layer.get_deriv(point);
            result.0 += noise.0;
            result.1[0] += noise.1[0];
            result.1[1] += noise.1[1];
            result.1[2] += noise.1[2];
        }
        result.0 *= i16::MAX as f64;
        result
    }
    pub fn try_reload(&mut self) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::io::BufRead;
        let mut hasher = DefaultHasher::new();
        let file = std::fs::File::open(&self.file_path).unwrap();
        let mut reader = std::io::BufReader::new(file);
        reader.fill_buf().unwrap().hash(&mut hasher);
        let hash = hasher.finish();
        if self.file_hash != hash {
            self.layers = load_layers_from_file(&std::path::Path::new(&self.file_path));
            self.file_hash = hash;
            return true;
        }
        return false;
    }
    pub fn tree_density_at(&self, point: na::Point3<f64>) -> f64 {
        let x = self.get(point, 254) / i16::MAX as f64 * 5.0 - 1.5;
        let x = 1.0 - x.powi(2);
        x.max(0.0).min(1.0)
    }
}

impl Default for PlanetProcGen {
    // Default layers configuration
    fn default() -> Self {
        let file_path = std::path::Path::new("./assets/planet.json");
        let layers = load_layers_from_file(&file_path);
        Self {
            layers,
            file_path: "./assets/planet.json".to_string(),
            file_hash: 0,
        }
    }
}

fn load_layers_from_file(path: &std::path::Path) -> Vec<Layer> {
    let file = std::fs::File::open(path).unwrap();
    let reader = std::io::BufReader::new(file);
    serde_json::from_reader(reader).unwrap()
}

unsafe fn fbm_3d_deriv<S: simdeez::Simd>(
    mut x: S::Vf32,
    mut y: S::Vf32,
    mut z: S::Vf32,
    lac: S::Vf32,
    gain: S::Vf32,
    seed: i32,
    octaves: u8,
) -> (S::Vf32, [S::Vf32; 3]) {
    let mut result = simdnoise::simplex::simplex_3d_deriv::<S>(x, y, z, seed);
    let mut amp = S::set1_ps(1.0);
    for _ in 1..octaves {
        x = S::mul_ps(x, lac);
        y = S::mul_ps(y, lac);
        z = S::mul_ps(z, lac);
        amp = S::mul_ps(amp, gain);
        let noise = simdnoise::simplex::simplex_3d_deriv::<S>(x, y, z, seed);
        result.0 = S::add_ps(S::mul_ps(noise.0, amp), result.0);
        result.1[0] += noise.1[0] * amp * lac;
        result.1[1] += noise.1[1] * amp * lac;
        result.1[2] += noise.1[2] * amp * lac;
    }
    result
}
