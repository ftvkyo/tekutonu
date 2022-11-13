use super::{chunk::Chunk, consts as c, types as t};


pub struct Region {
    chunks: ndarray::Array3<Chunk>,
}

impl Region {
    pub fn get_chunk(&self, loc: t::PointIntLocal) -> &Chunk {
        &self.chunks[loc]
    }

    pub fn get_chunk_mut(&mut self, loc: t::PointIntLocal) -> &mut Chunk {
        &mut self.chunks[loc]
    }

    pub fn set_chunk(&mut self, loc: t::PointIntLocal, chunk: Chunk) {
        self.chunks[loc] = chunk;
    }
}

impl Default for Region {
    fn default() -> Self {
        Self {
            chunks: ndarray::Array3::default((
                c::REGION_X_CHUNKS,
                c::REGION_Y_CHUNKS,
                c::REGION_Z_CHUNKS,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::block::BlockKind;

    #[test]
    fn accessing_block() {
        let reg = Region::default();
        let chunk = reg.get_chunk([1, 2, 3]);
        let block = chunk.get_block([1, 2, 3]);

        assert_eq!(block.kind, BlockKind::Air);
    }
}
