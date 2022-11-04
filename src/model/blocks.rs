use super::{consts as c, types as t};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind {
    Air,
    Stone,
}

#[derive(Clone, Copy)]
pub struct Block {
    pub kind: BlockKind,
}

impl Block {
    pub fn air() -> Self {
        Self {
            kind: BlockKind::Air,
        }
    }
}

#[derive(Clone)]
pub struct Chunk {
    blocks: [Block; c::CHUNK_TOTAL_BLOCKS],
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            blocks: [Block::air(); c::CHUNK_TOTAL_BLOCKS],
        }
    }

    const fn generate_index(location: t::PointInt) -> usize {
        let [x, y, z] = location;
        z * c::CHUNK_ONE_SLICE_BLOCKS + y * c::CHUNK_ONE_COLUMN_BLOCKS + x
    }

    pub fn get_block<'s>(&'s self, location: t::PointInt) -> &'s Block {
        &self.blocks[Self::generate_index(location)]
    }

    pub fn get_block_mut(&mut self, location: t::PointInt) -> &mut Block {
        &mut self.blocks[Self::generate_index(location)]
    }

    pub fn set_block(&mut self, location: t::PointInt, block: Block) {
        self.blocks[Self::generate_index(location)] = block;
    }
}

pub struct Region {
    chunks: Vec<Chunk>,
}

impl Region {
    pub fn new() -> Self {

        Self {
            chunks: vec![Chunk::new(); c::REGION_TOTAL_CHUNKS],
        }
    }

    const fn generate_index(location: t::PointInt) -> usize {
        let [x, y, z] = location;
        z * c::REGION_ONE_SLICE_CHUNKS + y * c::REGION_ONE_COLUMN_CHUNKS + x
    }

    pub fn get_chunk<'s>(&'s self, location: t::PointInt) -> &'s Chunk {
        &self.chunks[Self::generate_index(location)]
    }

    pub fn get_chunk_mut(&mut self, location: t::PointInt) -> &mut Chunk {
        &mut self.chunks[Self::generate_index(location)]
    }

    pub fn set_chunk(&mut self, location: t::PointInt, chunk: Chunk) {
        self.chunks[Self::generate_index(location)] = chunk;
    }
}
