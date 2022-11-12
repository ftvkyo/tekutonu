use super::{chunk::Chunk, consts as c, types as t};


pub struct Region {
    chunks: Vec<Chunk>,
}

impl Region {
    pub fn new() -> Self {
        Self {
            chunks: vec![Chunk::new(); c::REGION_TOTAL_CHUNKS],
        }
    }

    const fn generate_index(location: t::PointIntLocal) -> usize {
        let [x, y, z] = location;
        z * c::REGION_ONE_SLICE_CHUNKS + y * c::REGION_ONE_COLUMN_CHUNKS + x
    }

    pub fn get_chunk<'s>(&'s self, location: t::PointIntLocal) -> &'s Chunk {
        &self.chunks[Self::generate_index(location)]
    }

    pub fn get_chunk_mut(&mut self, location: t::PointIntLocal) -> &mut Chunk {
        &mut self.chunks[Self::generate_index(location)]
    }

    pub fn set_chunk(&mut self, location: t::PointIntLocal, chunk: Chunk) {
        self.chunks[Self::generate_index(location)] = chunk;
    }
}
