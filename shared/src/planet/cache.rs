// Based on https://github.com/Ralith/planetmap/blob/master/src/cache.rs

use crate::planet::chunk::Chunk;
use crate::planet::Face;

pub struct Slot {
    chunk: Chunk,
    /// Whether the slot is ready for reading
    ready: bool,
}

pub struct ChunkInfo {
    chunk: Chunk,
    index: Option<u32>,
    renderable: bool,
}

pub struct Cache {
    chunks: slab::Slab<Slot>,
    index: std::collections::HashMap<Chunk, u32>,
    pub transfer: Vec<Chunk>,
    pub render: Vec<Chunk>,
    max_depth: u8,
    pub used: Vec<bool>,
    pub capacity: usize,
}

impl Cache {
    pub fn new(max_depth: u8) -> Self {
        let capacity = slots_needed(max_depth) as usize;
        Cache {
            chunks: slab::Slab::with_capacity(capacity),
            index: std::collections::HashMap::with_capacity(capacity),
            transfer: Vec::with_capacity(capacity),
            render: Vec::with_capacity(capacity),
            max_depth: max_depth,
            used: vec![false; capacity],
            capacity,
        }
    }
    fn walk_inner(&mut self, chunk: ChunkInfo, viewpoints: &[na::Point3<f64>]) {
        if let Some(id) = chunk.index {
            self.used[id as usize] = true;
        } else {
            self.transfer.push(chunk.chunk);
        }
        let subdivide = chunk.chunk.depth < self.max_depth
            && viewpoints
                .iter()
                .any(|x| needs_subdivision(&chunk.chunk, x));
        if !subdivide {
            if chunk.renderable {
                self.render.push(chunk.chunk);
            }
            return;
        }
        let children = chunk.chunk.subdivide();
        let child_slots = [
            self.get(&children[0]),
            self.get(&children[1]),
            self.get(&children[2]),
            self.get(&children[3]),
        ];

        let children_renderable = chunk.renderable
            && child_slots
                .iter()
                .all(|slot| slot.map_or(false, |x| self.chunks[x as usize].ready));
        if chunk.renderable && !children_renderable {
            self.render.push(chunk.chunk);
        }

        for (&chunk, &index) in children.iter().zip(child_slots.iter()) {
            self.walk_inner(
                ChunkInfo {
                    chunk,
                    index,
                    renderable: children_renderable,
                },
                viewpoints,
            );
        }
    }

    pub fn update(&mut self) {
        let mut available = self.chunks.capacity() - self.chunks.len();
        let ids = self
            .used
            .iter()
            .cloned()
            .enumerate()
            .filter_map(|(id, used)| if used { None } else { Some(id) });
        for id in ids {
            if available >= self.transfer.len() {
                break;
            }
            if self.chunks.contains(id) && self.chunks[id as usize].ready {
                let old = self.chunks.remove(id);
                self.index.remove(&old.chunk);
                available += 1;
            }
        }
        self.transfer.truncate(available);
    }

    pub fn clear(&mut self) {
        self.transfer.clear();
        self.used = vec![false; self.capacity];
    }

    pub fn allocate(&mut self, chunk: Chunk) -> Option<u32> {
        if self.chunks.len() == self.chunks.capacity() {
            return None;
        }
        let id = self.chunks.insert(Slot {
            chunk,
            ready: false,
        }) as u32;
        let _old = self.index.insert(chunk, id);
        Some(id)
    }

    // Render query
    pub fn query(&mut self, viewpoints: &[na::Point3<f64>]) {
        self.render.clear();
        for face_chunk in Face::iter().map(Chunk::from_face) {
            let slot = self.get(&face_chunk);
            self.walk_inner(
                ChunkInfo {
                    chunk: face_chunk,
                    index: slot,
                    renderable: slot.map_or(false, |idx| self.chunks[idx as usize].ready),
                },
                viewpoints,
            );
        }
    }

    pub fn release(&mut self, slot: u32) {
        self.chunks[slot as usize].ready = true;
    }

    pub fn get(&self, chunk: &Chunk) -> Option<u32> {
        self.index.get(chunk).cloned()
    }
}

fn needs_subdivision(chunk: &Chunk, viewpoint: &na::Point3<f64>) -> bool {
    let max_half_angle = 1.0f64.atan2(10.0f64.sqrt());
    let center =
        na::Point3::from(chunk.coords.face.basis() * (chunk.origin_on_face().into_inner()));
    if center.coords.dot(&viewpoint.coords) < 0.0 {
        return false;
    }
    let distance = na::distance(&center, viewpoint);
    let half_angle = (chunk.edge_length() / 2.0).atan2(distance);
    half_angle >= max_half_angle
}

pub fn slots_needed(depth: u8) -> u32 {
    Face::iter()
        .map(Chunk::from_face)
        .map(|x| slots_needed_inner(&x, depth))
        .sum::<u32>()
}

fn slots_needed_inner(chunk: &Chunk, depth: u8) -> u32 {
    let viewpoint = na::Point3::from(na::Vector3::new(1.0, 1.0, 1.0).normalize());
    if chunk.depth == depth || !needs_subdivision(&chunk, &viewpoint) {
        return 1;
    }
    chunk
        .subdivide()
        .iter()
        .map(|x| slots_needed_inner(x, depth))
        .sum::<u32>()
        + 1
}
