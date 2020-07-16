use rand::SeedableRng;
use shared::planet::cache::Cache;
use shared::planet::chunk::Chunk;
use shared::planet::procgen::*;
use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};

pub const CHUNK_SAMPLES: u32 = 17;
pub const CHUNK_QUADS: u32 = CHUNK_SAMPLES - 1;
pub const NORMAL_SAMPLES: u32 = CHUNK_SAMPLES;
pub const CLOUDS_HEIGHT: f64 = 1_500.0;

pub struct ChunkData {
    pub heightmap: glium::texture::Texture2d,
    pub normalmap: glium::texture::Texture2d,
    pub trees: Vec<(usize, (f32, f32), f32)>,
}

pub struct Planet {
    pub radius: f64,
    pub procgen: PlanetProcGen,
    pub heightmaps: HashMap<u32, ChunkData>,
    pub terrain_textures: glium::texture::SrgbTexture2dArray,
    pub requests: Sender<(Chunk, u32)>,
    // Chunk, heightmap, normalmap
    pub output: Receiver<(Chunk, u32, Vec<i16>, Vec<i8>)>,
    pub surface_cache: Cache,
    pub water_cache: Cache,
    pub clouds_cache: Cache,
}

impl Planet {
    pub fn new(radius: f64, textures: glium::texture::SrgbTexture2dArray) -> Self {
        let (requests, requests_rx) = std::sync::mpsc::channel::<(Chunk, u32)>();
        let (output_tx, output) = std::sync::mpsc::channel();

        // We create this thread to prevent render blocking when loading new surface areas
        std::thread::spawn(move || {
            // We cannot move/clone procgen, so we have to recreate it here
            let procgen = PlanetProcGen::default();

            loop {
                let (chunk, slot) = requests_rx.recv().unwrap();
                // Capture samples
                let mut heights: Vec<i16> = Vec::with_capacity(CHUNK_SAMPLES.pow(2) as usize);
                for dir in chunk.samples(CHUNK_SAMPLES) {
                    let p = na::Point::from(dir.into_inner() * radius);
                    let h = procgen.get(p, chunk.depth) as i16;
                    heights.push(h);
                }

                let mut normals: Vec<i8> = Vec::with_capacity(CHUNK_SAMPLES.pow(2) as usize * 2);
                for dir in chunk.samples(NORMAL_SAMPLES) {
                    let dir = dir.into_inner();
                    let basis = shared::planet::Face::from_vector(&dir).basis();
                    let perp = basis.matrix().index((.., 1));
                    let x = dir.cross(&perp);
                    let y = dir.cross(&x);
                    let x_off = x * 1e-3;
                    let y_off = y * 1e-3;
                    let center = dir * radius;
                    //FIXME: Remove this `/ 4.0` hardcode. I have to find a way to do it better, meh
                    let x_0 = procgen.get(na::Point::from(center - x_off), chunk.depth) / 12.0;
                    let x_1 = procgen.get(na::Point::from(center + x_off), chunk.depth) / 12.0;
                    let y_0 = procgen.get(na::Point::from(center - y_off), chunk.depth) / 12.0;
                    let y_1 = procgen.get(na::Point::from(center + y_off), chunk.depth) / 12.0;

                    let dx = (x_1 - x_0) / (2.0 * 1e-3);
                    let dy = (y_1 - y_0) / (2.0 * 1e-3);

                    let normal_unit: na::Unit<na::Vector3<f64>> =
                        na::Unit::new_normalize(na::Vector3::new(-dx, -dy, 1.0));

                    normals.push((normal_unit.x * 127.0) as i8);
                    normals.push((normal_unit.y * 127.0) as i8);
                }

                output_tx.send((chunk, slot, heights, normals)).unwrap();
            }
        });

        Self {
            radius: radius,
            procgen: PlanetProcGen::default(),
            // Should be enough.
            heightmaps: HashMap::with_capacity(1024),
            terrain_textures: textures,
            requests,
            output,
            surface_cache: Cache::new(15),
            water_cache: Cache::new(11),
            clouds_cache: Cache::new(5),
        }
    }

    pub fn height_at(&self, dir: na::Vector3<f64>, depth: u8) -> f64 {
        let p = na::Point::from(dir);
        self.procgen.get(p, depth)
    }

    pub fn trees_at(&self, chunk: &Chunk) -> Vec<(usize, (f32, f32), f32)> {
        use rand::Rng;
        // That's a nice array right here
        let be_x = chunk.coords.coords.0.to_be_bytes();
        let be_y = chunk.coords.coords.1.to_be_bytes();
        let seed = [
            be_x[0],
            be_x[1],
            be_x[2],
            be_x[3],
            be_y[0],
            be_y[1],
            be_y[2],
            be_y[3],
            chunk.depth,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];
        let mut rng = rand::rngs::StdRng::from_seed(seed);
        let mut result = vec![];
        for i in 0..5 {
            let x: f32 = rng.gen_range(0.0, 1.0);
            let y: f32 = rng.gen_range(0.0, 1.0);
            let angle: f32 = rng.gen_range(-3.14, 3.14);
            result.push((0, (x, y), angle));
        }
        result
    }

    pub fn update_cache(&mut self, camera_position: na::Vector3<f64>) {
        let surface_height = self.radius + 32000.0;
        let mut surface_viewpoint = {
            if camera_position.norm() > surface_height {
                camera_position / surface_height
            } else {
                (camera_position / surface_height).normalize()
            }
        };

        if surface_viewpoint.magnitude() < 1.0 {
            surface_viewpoint = surface_viewpoint.normalize();
        }
        let water_viewpoint = camera_position / self.radius;
        let clouds_viewpoint = camera_position / (self.radius + CLOUDS_HEIGHT);

        self.surface_cache.query(&[surface_viewpoint.into()]);
        self.water_cache.query(&[water_viewpoint.into()]);
        self.clouds_cache.query(&[clouds_viewpoint.into()]);

        // Get chunks for a transfer
        self.surface_cache.update();
        self.water_cache.update();
        self.clouds_cache.update();
    }

    pub fn allocate_chunks(&mut self) {
        for chunk in self.surface_cache.transfer.clone() {
            let slot = self.surface_cache.allocate(chunk).unwrap();
            self.requests.send((chunk, slot)).unwrap();
        }
        for chunk in self.water_cache.transfer.clone() {
            let slot = self.water_cache.allocate(chunk).unwrap();
            self.water_cache.release(slot);
        }
        for chunk in self.clouds_cache.transfer.clone() {
            let slot = self.clouds_cache.allocate(chunk).unwrap();
            self.clouds_cache.release(slot);
        }
        self.surface_cache.clear();
        self.water_cache.clear();
        self.clouds_cache.clear();
    }
}
