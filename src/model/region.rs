use cgmath::{Point3, Vector3};
use tracing::instrument;

use super::{
    chunk::{Chunk, SurroundingChunks},
    consts as c,
    types as t,
};


pub struct Region {
    chunks: ndarray::Array3<Chunk>,
}

impl Region {
    pub fn get_chunk(&self, loc: impl Into<t::PointIntLocal>) -> &Chunk {
        let loc = loc.into();
        &self.chunks[[loc.ux(), loc.uy(), loc.uz()]]
    }

    pub fn set_chunk(&mut self, loc: impl Into<t::PointIntLocal>, chunk: Chunk) {
        let loc = loc.into();
        self.chunks[[loc.ux(), loc.uy(), loc.uz()]] = chunk;
    }

    #[instrument(skip_all)]
    pub fn recalculate_chunk_light(&mut self, loc: impl Into<t::PointIntLocal>) {
        let loc = loc.into();
        if loc.x() == 0
            || loc.x() >= c::REGION_X_CHUNKS as isize
            || loc.y() == 0
            || loc.y() >= c::REGION_Y_CHUNKS as isize
            || loc.z() == 0
            || loc.z() >= c::REGION_Z_CHUNKS as isize
        {
            panic!(
                "Can't recalculate light for a chunk on a face of the region: {}",
                loc
            );
        }

        let surrounding = SurroundingChunks::new([
            self.get_chunk(loc.with_x(loc.x() + 1)),
            self.get_chunk(loc.with_x(loc.x() - 1)),
            self.get_chunk(loc.with_y(loc.y() + 1)),
            self.get_chunk(loc.with_y(loc.y() - 1)),
            self.get_chunk(loc.with_z(loc.z() + 1)),
            self.get_chunk(loc.with_z(loc.z() - 1)),
        ]);

        let mut updated = self.get_chunk(loc).clone();
        updated.recalculate_light(surrounding);

        self.set_chunk(loc, updated);
    }

    pub fn get_render_data(
        &self,
        global_offset: Vector3<f32>,
    ) -> (Vec<Point3<f32>>, Vec<u8>, Vec<usize>) {
        let mut vertices = Vec::new();
        let mut lights = Vec::new();
        let mut indices = Vec::new();

        for (loc, chunk) in self.chunks.indexed_iter() {
            let (chunk_vertices, chunk_lights, chunk_indices) = chunk.get_render_data(
                global_offset
                    + Vector3::new(
                        loc.0 as f32 * c::CHUNK_X_BLOCKS as f32,
                        loc.1 as f32 * c::CHUNK_Y_BLOCKS as f32,
                        loc.2 as f32 * c::CHUNK_Z_BLOCKS as f32,
                    ),
            );

            vertices.extend(chunk_vertices);
            lights.extend(chunk_lights);
            indices.extend(chunk_indices);
        }

        (vertices, lights, indices)
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
        let chunk = reg.get_chunk([1isize, 2, 3]);
        let block = chunk.get_block([1isize, 2, 3]);

        assert_eq!(block.kind, BlockKind::Air);
    }
}
