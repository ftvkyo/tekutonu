use super::{chunk::Chunk, consts as c, types as t};


pub struct Region {
    chunks: ndarray::Array3<Chunk>,
}

impl Region {
    pub fn new() -> Self {
        Self {
            chunks: ndarray::Array3::default((
                c::REGION_X_CHUNKS,
                c::REGION_Y_CHUNKS,
                c::REGION_Z_CHUNKS,
            )),
        }
    }

    pub fn get_chunk<'s>(&'s self, loc: t::PointIntLocal) -> &'s Chunk {
        &self.chunks[loc]
    }

    pub fn get_chunk_mut(&mut self, loc: t::PointIntLocal) -> &mut Chunk {
        &mut self.chunks[loc]
    }

    pub fn set_chunk(&mut self, loc: t::PointIntLocal, chunk: Chunk) {
        self.chunks[loc] = chunk;
    }
}

#[cfg(test)]
mod tests {
    use crate::model::block::BlockKind;

    use super::*;

    #[test]
    fn accessing_block() {
        let reg = Region::new();
        let chunk = reg.get_chunk([1, 2, 3]);
        let block = chunk.get_block([1, 2, 3]);

        assert_eq!(block.kind, BlockKind::Air);
    }
}
