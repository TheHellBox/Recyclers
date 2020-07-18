pub enum NoiseType {
    Simplex,
    Fbm { lac: f64, gain: f64, octaves: u8 },
}

impl NoiseType {
    pub fn get(&self, coords: na::Vector3<f64>) -> f64 {
        //use core::arch::x86_64::_mm_set1_pd;
        //use simdeez::Simd;
        let x = coords.x;
        let y = coords.y;
        let z = coords.z;

        //FIXME: remove that hardcode. Use server seed
        let seed = 1234;

        match self {
            Self::Simplex => unsafe {
                // TODO: Finish simd implementation
                /*if is_x86_feature_detected!("avx2") {
                    unsafe{
                        let x = _mm256_set1_ps(x);
                        let y = _mm256_set1_ps(y);
                        let z = _mm256_set1_ps(z);
                        let freq = _mm256_set1_ps(1.0);
                        let r: simdeez::avx2::F32x8 = simdnoise::avx2::simplex_3d(x, y, z, seed);
                        r
                    }
                }
                else{
                    simdnoise::scalar::simplex_3d(x, y, z, seed)
                }*/
                simdnoise::scalar::simplex_3d(x as f32, y as f32, z as f32, seed) as f64
            },
            Self::Fbm { lac, gain, octaves } => unsafe {
                /*if is_x86_feature_detected!("sse4.1") {
                    let x = _mm_set1_pd(x);
                    let y = _mm_set1_pd(y);
                    let z = _mm_set1_pd(z);
                    let lac = _mm_set1_pd(*lac);
                    let gain = _mm_set1_pd(*gain);

                    let s = simdeez::sse41::F64x2(simdnoise::sse41::fbm_3d_f64(x, y, z, lac, gain, *octaves, seed as i64));
                    let mut r: f64 = 0.0;
                    simdeez::sse41::Sse41::storeu_pd(&mut r, s);
                    r
                }
                else{*/
                simdnoise::scalar::fbm_3d_f64(x, y, z, *lac, *gain, *octaves, seed as i64)
                //}
            },
        }
    }
}

pub enum LayerType {
    Noise {
        noise: NoiseType,
        post: Box<dyn Fn(f64, f64) -> f64 + Send + Sync>,
        frequency: f64,
        depth: f64,
    },
    Value(f64)
}

impl LayerType {
    pub fn get(&self, point: na::Point3<f64>, mask: f64) -> f64 {
        match self {
            LayerType::Noise {
                noise,
                frequency,
                post,
                depth,
            } => post(noise.get((point.coords * *frequency).into()), mask) * *depth,
            LayerType::Value(v) => *v * mask
        }
    }
}

pub struct Layer {
    pub layer_type: LayerType,
    pub mask: Option<usize>,
    pub min_lod: u8,
}

// Samples collection
pub struct PlanetProcGen {
    pub layers: Vec<Layer>,
}

impl PlanetProcGen {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self { layers: layers }
    }
    pub fn get(&self, point: na::Point3<f64>, depth: u8) -> f64 {
        let mut result = 0.0;
        let mut cache = Vec::with_capacity(self.layers.len());
        for layer in &self.layers {
            if depth < layer.min_lod {
                continue;
            }
            let mut m = 1.0f64;
            if let Some(mask) = &layer.mask {
                m = cache[*mask];
            }
            let h = layer.layer_type.get(point, m);
            cache.push(h);
            result += h;
        }
        result * i16::MAX as f64
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
        let mut layers = Vec::new();

        let _ = layers.push(Layer {
            layer_type: LayerType::Noise {
                noise: NoiseType::Fbm {
                    octaves: 6,
                    gain: 0.5,
                    lac: std::f64::consts::PI * 2.0 / 3.0,
                },
                frequency: 0.0000005,
                post: Box::new(|x, _m| x),
                depth: 3.5,
            },
            mask: None,
            min_lod: 0,
        });
        //Mountains
        let _ = layers.push(Layer {
            layer_type: LayerType::Noise {
                noise: NoiseType::Fbm {
                    octaves: 6,
                    gain: 0.5,
                    lac: std::f64::consts::PI * 2.0 / 3.0,
                },
                frequency: 0.000001,
                post: Box::new(|x, m| (1.0 - x.abs() * 100.0).powi(6) * m),
                depth: 20.0,
            },
            mask: Some(0),
            min_lod: 0,
        });

        // Rivers
        let _ = layers.push(Layer {
            layer_type: LayerType::Noise {
                noise: NoiseType::Fbm {
                    octaves: 4,
                    gain: 0.8,
                    lac: std::f64::consts::PI * 2.0 / 3.0,
                },
                frequency: 0.000002,
                post: Box::new(|x, _m| (-(1.0 - x.abs() * 100.0).powi(25)).min(0.0)),
                depth: 0.05,
            },
            mask: Some(0),
            min_lod: 0,
        });

        let _ = layers.push(Layer {
            layer_type: LayerType::Noise {
                noise: NoiseType::Fbm {
                    octaves: 8,
                    gain: 0.5,
                    lac: std::f64::consts::PI * 2.0 / 3.0,
                },
                frequency: 0.001,
                post: Box::new(|x, m| x * m.max(0.0)),
                depth: 0.2,
            },
            mask: None,
            min_lod: 0,
        });
        let _ = layers.push(Layer {
            layer_type: LayerType::Noise {
                noise: NoiseType::Fbm {
                    octaves: 3,
                    gain: 0.5,
                    lac: std::f64::consts::PI * 2.0 / 3.0,
                },
                frequency: 0.0005,
                post: Box::new(|x, m| (0.1 - x.max(0.0)).powi(4)),
                depth: 0.6,
            },
            mask: None,
            min_lod: 0,
        });

        Self::new(layers)
    }
}
