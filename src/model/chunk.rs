use std::collections::{HashSet, VecDeque};

use cgmath::{Point3, Vector3};
use tracing::instrument;

use super::{
    block::{Block, BlockKind},
    consts as c,
    types as t,
};

type ChunkBlockData<Data> = [[[Data; c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS]; c::CHUNK_X_BLOCKS];


pub enum Axis {
    X,
    Y,
    Z,
}

pub enum Sign {
    Positive,
    Negative,
}


#[derive(Clone, Copy)]
pub struct SurroundingChunks<'a> {
    chunks: [&'a Chunk; 6],
}

impl<'a> SurroundingChunks<'a> {
    pub fn new(
        chunks: [&'a Chunk; 6],
    ) -> Self {
        Self {
            chunks,
        }
    }

    pub fn get(&self, axis: Axis, sign: Sign) -> &'a Chunk {
        let index = match (axis, sign) {
            (Axis::X, Sign::Positive) => 0,
            (Axis::X, Sign::Negative) => 1,
            (Axis::Y, Sign::Positive) => 2,
            (Axis::Y, Sign::Negative) => 3,
            (Axis::Z, Sign::Positive) => 4,
            (Axis::Z, Sign::Negative) => 5,
        };
        self.chunks[index]
    }

    /// Get local light level for a location on one of inner chunk's faces based on the surrounding local light data.
    ///
    /// If the location is adjacent to multiple surrounding chunks,
    /// the light level is the maximum of the surrounding light levels.
    ///
    /// For blocks that are not on the inner chunk's faces, return 0.
    pub fn inner_light_local(&self, loc: t::PointIntLocal) -> u8 {
        let mut light = 0;

        if loc.x() == 0 {
            light = light.max(self.get(Axis::X, Sign::Negative).light_local[c::CHUNK_X_BLOCKS - 1][loc.uy()][loc.uz()] as i8 - 1);
        }

        if loc.x() == c::CHUNK_X_BLOCKS as isize - 1 {
            light = light.max(self.get(Axis::X, Sign::Positive).light_local[0][loc.uy()][loc.uz()] as i8 - 1);
        }

        if loc.y() == 0 {
            light = light.max(self.get(Axis::Y, Sign::Negative).light_local[loc.ux()][c::CHUNK_Y_BLOCKS - 1][loc.uz()] as i8 - 1);
        }

        if loc.y() == c::CHUNK_Y_BLOCKS as isize - 1 {
            light = light.max(self.get(Axis::Y, Sign::Positive).light_local[loc.ux()][0][loc.uz()] as i8 - 1);
        }

        if loc.z() == 0 {
            light = light.max(self.get(Axis::Z, Sign::Negative).light_local[loc.ux()][loc.uy()][c::CHUNK_Z_BLOCKS - 1] as i8 - 1);
        }

        if loc.z() == c::CHUNK_Z_BLOCKS as isize - 1 {
            light = light.max(self.get(Axis::Z, Sign::Positive).light_local[loc.ux()][loc.uy()][0] as i8 - 1);
        }

        light as u8
    }

    /// Get sky light level for a location on one of the inner chunk's faces based on the surrounding sky light data.
    ///
    /// If the location is adjacent to multiple surrounding chunks,
    /// the light level is the maximum of the surrounding light levels.
    ///
    /// Sky light propagates down without weakening.
    ///
    /// For blocks that are not on the inner chunk's faces, return 0.
    pub fn inner_light_sky(&self, loc: t::PointIntLocal) -> u8 {
        let mut light = 0;

        if loc.x() == 0 {
            light = light.max(self.get(Axis::X, Sign::Negative).light_local[c::CHUNK_X_BLOCKS - 1][loc.uy()][loc.uz()] as i8 - 1);
        }

        if loc.x() == c::CHUNK_X_BLOCKS as isize - 1 {
            light = light.max(self.get(Axis::X, Sign::Positive).light_local[0][loc.uy()][loc.uz()] as i8 - 1);
        }

        if loc.y() == 0 {
            light = light.max(self.get(Axis::Y, Sign::Negative).light_local[loc.ux()][c::CHUNK_Y_BLOCKS - 1][loc.uz()] as i8 - 1);
        }

        if loc.y() == c::CHUNK_Y_BLOCKS as isize - 1 {
            // Not stubtracting 1 here because sky light level doesn't decrease when going down
            light = light.max(self.get(Axis::Y, Sign::Positive).light_local[loc.ux()][0][loc.uz()] as i8);
        }

        if loc.z() == 0 {
            light = light.max(self.get(Axis::Z, Sign::Negative).light_local[loc.ux()][loc.uy()][c::CHUNK_Z_BLOCKS - 1] as i8 - 1);
        }

        if loc.z() == c::CHUNK_Z_BLOCKS as isize - 1 {
            light = light.max(self.get(Axis::Z, Sign::Positive).light_local[loc.ux()][loc.uy()][0] as i8 - 1);
        }

        light as u8
    }
}


#[derive(Clone)]
pub struct Chunk {
    blocks: ChunkBlockData<Block>,
    light_sky: ChunkBlockData<u8>,
    light_local: ChunkBlockData<u8>,
    light_sources: HashSet<t::PointIntLocal>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            blocks: [[[Block::air(); c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS]; c::CHUNK_X_BLOCKS],
            light_sky: [[[15; c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS]; c::CHUNK_X_BLOCKS],
            light_local: [[[0; c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS]; c::CHUNK_X_BLOCKS],
            light_sources: HashSet::new(),
        }
    }
}

impl Chunk {
    pub fn get_block(&self, loc: impl Into<t::PointIntLocal>) -> &Block {
        let loc = loc.into();
        &self.blocks[loc.ux()][loc.uy()][loc.uz()]
    }

    pub fn set_block(&mut self, loc: impl Into<t::PointIntLocal>, block: Block) {
        let loc = loc.into();
        let block_place = &mut self.blocks[loc.ux()][loc.uy()][loc.uz()];
        if let BlockKind::Light { .. } = block_place.kind {
            self.light_sources.remove(&loc);
        }
        *block_place = block;
        if let BlockKind::Light { .. } = block_place.kind {
            self.light_sources.insert(loc);
        }
    }

    pub fn get_light(&self, loc: impl Into<t::PointIntLocal>) -> u8 {
        let loc = loc.into();
        self.light_local[loc.ux()][loc.uy()][loc.uz()]
            .max(self.light_sky[loc.ux()][loc.uy()][loc.uz()])
    }

    #[instrument(skip_all)]
    fn recalculate_light_sky(&mut self, around: SurroundingChunks) {
        for x in 0..c::CHUNK_X_BLOCKS {
            for y in 0..c::CHUNK_Y_BLOCKS {
                for z in 0..c::CHUNK_Z_BLOCKS {
                    if self.blocks[x][y][z].kind == BlockKind::Air {
                        self.light_sky[x][y][z] = around.inner_light_sky([x, y, z].into());
                    } else {
                        self.light_sky[x][y][z] = 0;
                    }
                }
            }
        }

        let mut queue = VecDeque::<t::PointIntLocal>::new();

        for x in 0..c::CHUNK_X_BLOCKS {
            for z in 0..c::CHUNK_Z_BLOCKS {
                let loc = [x, c::CHUNK_Y_BLOCKS - 2, z].into();
                if self.get_block(loc).kind == BlockKind::Air {
                    queue.push_back(loc);
                }
            }
        }

        while let Some(loc) = queue.pop_front() {
            let light = self.light_sky[loc.ux()][loc.uy()][loc.uz()];
            if light == 0 {
                continue;
            }

            for dir in &c::ADJACENCY {
                let loc2 = loc + dir;
                if !loc2.is_in_chunk() {
                    // Location is outside of the chunk boundaries
                    continue;
                }
                if self.get_block(loc2).kind != BlockKind::Air {
                    // The block is not air
                    continue;
                }
                let light2 = &mut self.light_sky[loc2.ux()][loc2.uy()][loc2.uz()];
                if *light2 < light && loc2.y() < loc.y() {
                    *light2 = light;
                    queue.push_back(loc2);
                } else if *light2 < light - 1 {
                    *light2 = light - 1;
                    queue.push_back(loc2);
                }
            }
        }
    }

    #[instrument(skip_all)]
    fn recalculate_light_local(&mut self, around: SurroundingChunks) {
        for x in 0..c::CHUNK_X_BLOCKS {
            for y in 0..c::CHUNK_Y_BLOCKS {
                for z in 0..c::CHUNK_Z_BLOCKS {
                    if self.blocks[x][y][z].kind == BlockKind::Air {
                        self.light_local[x][y][z] = around.inner_light_local([x, y, z].into());
                    } else {
                        self.light_local[x][y][z] = 0;
                    }
                }
            }
        }

        for s in self.light_sources.iter() {
            let block = self.get_block(*s);
            match block.kind {
                BlockKind::Light { brightness } => {
                    self.light_local[s.ux()][s.uy()][s.uz()] = brightness;
                },
                _ => panic!("Light source is not a light block"),
            }
        }

        let mut queue = VecDeque::<t::PointIntLocal>::new();

        queue.extend(self.light_sources.iter());

        for x in 0..c::CHUNK_X_BLOCKS {
            for y in 0..c::CHUNK_Y_BLOCKS {
                for z in 0..c::CHUNK_Z_BLOCKS {
                    let loc: t::PointIntLocal = [x, y, z].into();
                    if loc.is_on_chunk_face() {
                        if self.get_block(loc).kind == BlockKind::Air {
                            queue.push_back(loc);
                        }
                    }
                }
            }
        }

        while let Some(loc) = queue.pop_front() {
            let light = self.light_local[loc.ux()][loc.uy()][loc.uz()];
            if light == 0 {
                continue;
            }

            for dir in &c::ADJACENCY {
                let loc2 = loc + dir;
                if !loc2.is_in_chunk() {
                    // Location is outside of the chunk boundaries
                    continue;
                }
                if self.get_block(loc2).kind != BlockKind::Air {
                    // The block is not air
                    continue;
                }
                let light2 = &mut self.light_local[loc2.ux()][loc2.uy()][loc2.uz()];
                if *light2 < light - 1 {
                    *light2 = light - 1;
                    queue.push_back(loc2);
                }
            }
        }
    }

    #[instrument(skip_all)]
    pub fn recalculate_light(&mut self, around: SurroundingChunks) {
        self.recalculate_light_sky(around);
        self.recalculate_light_local(around);
    }

    fn assemble_faces_with_light(&self) -> Vec<([Point3<f32>; 4], u8)> {
        let mut faces = Vec::<([Point3<f32>; 4], u8)>::new();

        for x in 0..c::CHUNK_X_BLOCKS {
            let fx = x as f32;
            for y in 0..c::CHUNK_Y_BLOCKS {
                let fy = y as f32;
                for z in 0..c::CHUNK_Z_BLOCKS {
                    let fz = z as f32;
                    let loc = t::PointIntLocal::from([x, y, z]);
                    let offset = Vector3::new(fx, fy, fz);
                    let block = self.get_block(loc);
                    if block.kind == BlockKind::Solid {
                        faces.extend(
                            c::BLOCK_FACES
                                .iter()
                                .enumerate()
                                .map(|(i, face)| {
                                    let loc2 = loc + &c::ADJACENCY[i];
                                    let light = if loc2.is_in_chunk() {
                                        self.get_light(loc2)
                                    } else {
                                        1
                                    };

                                    (face.map(|p| p + offset), light)
                                })
                        );
                    }
                }
            }
        }

        faces
    }

    pub fn get_render_data(
        &self,
        global_offset: Vector3<f32>,
    ) -> (Vec<Point3<f32>>, Vec<u8>, Vec<usize>) {
        let faces = self.assemble_faces_with_light();

        let add_offset = |p: Point3<f32>| p + global_offset;

        // Vertices
        let mut vs = vec![];
        // Light levels
        let mut ls = vec![];
        // Indices
        let mut is = vec![];

        for face in faces {
            let i = vs.len();
            // Four vertices
            vs.extend(face.0.into_iter().map(add_offset));
            let light = face.1;
            // TODO: calculate based on the light level of the block in front of us
            ls.extend([
                light, // For vertex 0
                light, // For vertex 1
                light, // For vertex 2
                light, // For vertex 3
            ]);
            // Two triangles
            is.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
        }

        (vs, ls, is)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod assemble_faces {
        use super::*;

        #[test]
        fn empty() {
            let chunk = Chunk::default();

            let ts = chunk.assemble_faces_with_light();

            assert_eq!(ts.len(), 0, "there should be zero faces generated");
        }

        #[test]
        fn one() {
            let mut chunk = Chunk::default();
            chunk.set_block([0isize, 0, 0], Block { kind: BlockKind::Solid });

            let ts = chunk.assemble_faces_with_light();

            assert_eq!(ts.len(), 6, "there should be 6 faces generated");
        }

        #[test]
        fn two() {
            let mut chunk = Chunk::default();
            chunk.set_block([0isize, 0, 0], Block { kind: BlockKind::Solid });
            chunk.set_block([0isize, 0, 1], Block { kind: BlockKind::Solid });

            let ts = chunk.assemble_faces_with_light();

            assert_eq!(ts.len(), 12, "there should be 12 faces generated");
        }
    }
}
