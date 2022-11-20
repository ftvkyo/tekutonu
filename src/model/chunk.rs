use std::collections::{HashSet, VecDeque};

use cgmath::{Point3, Vector3};
use tracing::instrument;

use super::{
    block::{Block, BlockKind},
    consts as c,
    types as t,
};

type ChunkBlockData<Data> = [[[Data; c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS]; c::CHUNK_X_BLOCKS];


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjacentDirection {
    XPos = 0,
    XNeg,
    YPos,
    YNeg,
    ZPos,
    ZNeg,
}

impl From<t::PointIntLocal> for AdjacentDirection {
    fn from(loc: t::PointIntLocal) -> Self {
        let x_pos = loc.x() >= c::CHUNK_X_BLOCKS as isize;
        let x_neg = loc.x() < 0;
        let y_pos = loc.y() >= c::CHUNK_Y_BLOCKS as isize;
        let y_neg = loc.y() < 0;
        let z_pos = loc.z() >= c::CHUNK_Z_BLOCKS as isize;
        let z_neg = loc.z() < 0;

        match (x_pos, x_neg, y_pos, y_neg, z_pos, z_neg) {
            (true, false, false, false, false, false) => Self::XPos,
            (false, true, false, false, false, false) => Self::XNeg,
            (false, false, true, false, false, false) => Self::YPos,
            (false, false, false, true, false, false) => Self::YNeg,
            (false, false, false, false, true, false) => Self::ZPos,
            (false, false, false, false, false, true) => Self::ZNeg,
            _ => panic!("Location is not in an adjacent chunk: {:?}", loc),
        }
    }
}


pub trait ChunkAdjacent {
    /// Trait methods should make sure the locaton is valid.
    fn assert_location_valid(loc: t::PointIntLocal)
    where
        Self: Sized,
    {
        debug_assert!(
            loc.is_on_chunk_face(),
            "Location {} is not on chunk face",
            loc
        );
    }

    /// Acquire the block at `loc`
    fn get_block(&self, loc: t::PointIntLocal) -> &Block;

    /// Acquire the local light level at `loc`
    fn get_light_local(&self, loc: t::PointIntLocal) -> u8;

    /// Acquire the sky light level at `loc`
    fn get_light_sky(&self, loc: t::PointIntLocal) -> u8;
}


pub struct ChunkEmpty {
    only_block: Block,
    light_local: u8,
    light_sky: u8,
}

impl ChunkEmpty {
    pub fn new(light_sky: u8) -> Self {
        Self {
            only_block: Block::air(),
            light_local: 0,
            light_sky,
        }
    }
}

impl ChunkAdjacent for ChunkEmpty {
    fn get_block(&self, loc: t::PointIntLocal) -> &Block {
        Self::assert_location_valid(loc);
        &self.only_block
    }

    fn get_light_local(&self, loc: t::PointIntLocal) -> u8 {
        Self::assert_location_valid(loc);
        self.light_local
    }

    fn get_light_sky(&self, loc: t::PointIntLocal) -> u8 {
        Self::assert_location_valid(loc);
        self.light_sky
    }
}

impl ChunkAdjacent for Chunk {
    fn get_block(&self, loc: t::PointIntLocal) -> &Block {
        Self::assert_location_valid(loc);
        self.get_block(loc)
    }

    fn get_light_local(&self, loc: t::PointIntLocal) -> u8 {
        Self::assert_location_valid(loc);
        self.get_light_local(loc)
    }

    fn get_light_sky(&self, loc: t::PointIntLocal) -> u8 {
        Self::assert_location_valid(loc);
        self.get_light_sky(loc)
    }
}


#[derive(Clone, Copy)]
pub struct SurroundingChunks<'a> {
    chunks: [&'a dyn ChunkAdjacent; 6],
}

impl<'a> SurroundingChunks<'a> {
    pub fn new(chunks: [&'a dyn ChunkAdjacent; 6]) -> Self {
        Self { chunks }
    }

    pub fn get_chunk_for_direction(&self, direction: &AdjacentDirection) -> &'a dyn ChunkAdjacent {
        self.chunks[*direction as usize]
    }

    pub fn get_chunk_of_location(
        &self,
        loc: t::PointIntLocal,
    ) -> (&'a dyn ChunkAdjacent, AdjacentDirection) {
        let direction = AdjacentDirection::from(loc);
        (self.get_chunk_for_direction(&direction), direction)
    }

    /// Get local light level for a location on one of inner chunk's faces based
    /// on the surrounding local light data.
    ///
    /// If the location is adjacent to multiple surrounding chunks,
    /// the light level is the maximum of the surrounding light levels.
    ///
    /// For blocks that are not on the inner chunk's faces, return 0.
    pub fn inner_light_local(&self, loc: t::PointIntLocal) -> u8 {
        if !loc.is_on_chunk_face() {
            return 0;
        }

        let mut light = 0;

        for adjacent in c::ADJACENCY {
            let loc_adjacent = loc + &adjacent;
            if !loc_adjacent.is_in_chunk() {
                let (chunk_adjacent, _) = self.get_chunk_of_location(loc_adjacent);
                let light_adjacent = chunk_adjacent.get_light_local(loc_adjacent.localize());
                light = light_adjacent.max(light + 1) - 1;
            }
        }

        light
    }

    /// Get sky light level for a location on one of the inner chunk's faces
    /// based on the surrounding sky light data.
    ///
    /// If the location is adjacent to multiple surrounding chunks,
    /// the light level is the maximum of the surrounding light levels.
    ///
    /// Sky light propagates down without weakening.
    ///
    /// For blocks that are not on the inner chunk's faces, return 0.
    pub fn inner_light_sky(&self, loc: t::PointIntLocal) -> u8 {
        if !loc.is_on_chunk_face() {
            return 0;
        }

        let mut light = 0;

        for adjacent in c::ADJACENCY {
            let loc_adjacent = loc + &adjacent;
            if !loc_adjacent.is_in_chunk() {
                let (chunk_adjacent, direction) = self.get_chunk_of_location(loc_adjacent);
                let light_adjacent = chunk_adjacent.get_light_sky(loc_adjacent.localize());

                if direction == AdjacentDirection::YPos {
                    light = light_adjacent.max(light);
                } else {
                    light = light_adjacent.max(light + 1) - 1;
                }
            }
        }

        light
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
            light_sky: [[[c::LIGHT_MAX as u8; c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS];
                c::CHUNK_X_BLOCKS],
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

    pub fn get_light_local(&self, loc: impl Into<t::PointIntLocal>) -> u8 {
        let loc = loc.into();
        self.light_local[loc.ux()][loc.uy()][loc.uz()]
    }

    pub fn get_light_sky(&self, loc: impl Into<t::PointIntLocal>) -> u8 {
        let loc = loc.into();
        self.light_sky[loc.ux()][loc.uy()][loc.uz()]
    }

    #[instrument(skip_all)]
    fn recalculate_light_sky(&mut self, around: SurroundingChunks) {
        let mut updated = VecDeque::<t::PointIntLocal>::new();

        // Collect light data from surrounding chunks and populate the queue
        for x in 0..c::CHUNK_X_BLOCKS {
            for y in 0..c::CHUNK_Y_BLOCKS {
                for z in 0..c::CHUNK_Z_BLOCKS {
                    if self.blocks[x][y][z].is_transparent() {
                        self.light_sky[x][y][z] = around.inner_light_sky([x, y, z].into());
                    } else {
                        self.light_sky[x][y][z] = 0;
                    }

                    if self.light_sky[x][y][z] != 0 {
                        updated.push_back([x, y, z].into());
                    }
                }
            }
        }

        while let Some(loc) = updated.pop_front() {
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
                if !self.get_block(loc2).is_transparent() {
                    // The block is not transparent, so light cannot pass through it
                    continue;
                }
                let light2 = &mut self.light_sky[loc2.ux()][loc2.uy()][loc2.uz()];
                if *light2 < light && loc2.y() < loc.y() {
                    *light2 = light;
                    updated.push_back(loc2);
                } else if *light2 < light - 1 {
                    *light2 = light - 1;
                    updated.push_back(loc2);
                }
            }
        }
    }

    #[instrument(skip_all)]
    fn recalculate_light_local(&mut self, around: SurroundingChunks) {
        let mut updated = VecDeque::<t::PointIntLocal>::new();

        // Collect light data from the surrounding chunks and populate the queue
        for x in 0..c::CHUNK_X_BLOCKS {
            for y in 0..c::CHUNK_Y_BLOCKS {
                for z in 0..c::CHUNK_Z_BLOCKS {
                    if self.blocks[x][y][z].is_transparent() {
                        self.light_local[x][y][z] = around.inner_light_local([x, y, z].into());
                    } else {
                        self.light_local[x][y][z] = 0;
                    }

                    if self.light_local[x][y][z] != 0 {
                        updated.push_back([x, y, z].into());
                    }
                }
            }
        }

        for s in self.light_sources.iter() {
            let block = self.get_block(*s);
            match block.kind {
                BlockKind::Light { brightness } => {
                    self.light_local[s.ux()][s.uy()][s.uz()] = brightness;
                    updated.push_back(*s);
                },
                _ => panic!("Light source is not a light block"),
            }
        }

        while let Some(loc) = updated.pop_front() {
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
                if !self.get_block(loc2).is_transparent() {
                    // The block is not air
                    continue;
                }
                let light2 = &mut self.light_local[loc2.ux()][loc2.uy()][loc2.uz()];
                if *light2 < light - 1 {
                    *light2 = light - 1;
                    updated.push_back(loc2);
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
                    if !block.is_transparent() {
                        faces.extend(c::BLOCK_FACES.iter().enumerate().filter_map(|(i, face)| {
                            let loc2 = loc + &c::ADJACENCY[i];

                            // TODO: make it work on chunk borders
                            let light = if loc2.is_in_chunk() {
                                let adjacent_block = self.get_block(loc2);
                                if !adjacent_block.is_transparent() {
                                    return None;
                                }

                                let local = self.get_light_local(loc2);
                                let sky = self.get_light_sky(loc2);
                                sky.max(local)
                            } else {
                                0
                            };

                            Some((face.map(|p| p + offset), light))
                        }));
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
            chunk.set_block(
                [0isize, 0, 0],
                Block {
                    kind: BlockKind::Solid,
                },
            );

            let ts = chunk.assemble_faces_with_light();

            assert_eq!(ts.len(), 6, "there should be 6 faces generated");
        }

        #[test]
        fn two() {
            let mut chunk = Chunk::default();
            chunk.set_block(
                [0isize, 0, 0],
                Block {
                    kind: BlockKind::Solid,
                },
            );
            chunk.set_block(
                [0isize, 0, 1],
                Block {
                    kind: BlockKind::Solid,
                },
            );

            let ts = chunk.assemble_faces_with_light();

            assert_eq!(ts.len(), 12, "there should be 12 faces generated");
        }
    }
}
