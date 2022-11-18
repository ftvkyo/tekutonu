use tracing::instrument;

use super::{chunk::{Chunk, SurroundingChunks}, consts as c};


pub struct Region {
    chunks: ndarray::Array3<Chunk>,
}

impl Region {
    pub fn get_chunk(&self, loc: impl Into<[usize; 3]>) -> &Chunk {
        &self.chunks[loc.into()]
    }

    pub fn set_chunk(&mut self, loc: impl Into<[usize; 3]>, chunk: Chunk) {
        self.chunks[loc.into()] = chunk;
    }

    #[instrument(skip_all)]
    pub fn recalculate_chunk_light(&mut self, loc: impl Into<[usize; 3]>) {
        let loc = loc.into();

        if loc[0] == 0 || loc[0] >= c::REGION_X_CHUNKS
            || loc[1] == 0 || loc[1] >= c::REGION_Y_CHUNKS
            || loc[2] == 0 || loc[2] >= c::REGION_Z_CHUNKS
        {
            panic!("Can't recalculate light for a chunk on a face of the region: {:?}", loc);
        }

        let surrounding = SurroundingChunks::new([
            // TODO: actually calculate
            self.get_chunk([2, 1, 1]),
            self.get_chunk([0, 1, 1]),
            self.get_chunk([1, 2, 1]),
            self.get_chunk([1, 0, 1]),
            self.get_chunk([1, 1, 2]),
            self.get_chunk([1, 1, 0]),
        ]);

        let mut updated = self.get_chunk(loc).clone();
        updated.recalculate_light(surrounding);

        self.set_chunk(loc, updated);
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
        let block = chunk.get_block([1isize, 2, 3]);

        assert_eq!(block.kind, BlockKind::Air);
    }
}
