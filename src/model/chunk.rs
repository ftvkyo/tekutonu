use super::{
    block::{Block, BlockKind},
    consts as c,
    types as t,
};

#[derive(Clone, Default)]
pub struct Chunk {
    blocks: [[[Block; c::CHUNK_Z_BLOCKS]; c::CHUNK_Y_BLOCKS]; c::CHUNK_X_BLOCKS],
}

impl Chunk {
    pub fn get_block<'s>(&'s self, loc: t::PointIntLocal) -> &'s Block {
        &self.blocks[loc[0]][loc[1]][loc[2]]
    }

    pub fn get_block_mut(&mut self, loc: t::PointIntLocal) -> &mut Block {
        &mut self.blocks[loc[0]][loc[1]][loc[2]]
    }

    pub fn set_block(&mut self, loc: t::PointIntLocal, block: Block) {
        self.blocks[loc[0]][loc[1]][loc[2]] = block;
    }

    fn assemble_faces(&self) -> Vec<[t::PointIntGlobal; 4]> {
        let mut faces = Vec::<[t::PointIntGlobal; 4]>::new();

        for x in 0..c::CHUNK_X_BLOCKS {
            for y in 0..c::CHUNK_Y_BLOCKS {
                for z in 0..c::CHUNK_Z_BLOCKS {
                    let block = self.get_block([x, y, z]);
                    if block.kind == BlockKind::Stone {
                        for face in c::BLOCK_FACES {
                            let face = face.map(|v| {
                                [
                                    v[0] + x as i64,
                                    v[1] + y as i64,
                                    v[2] + z as i64,
                                ]
                            });
                            faces.push(face);
                        }
                    }
                }
            }
        }

        faces
    }

    pub fn get_render_data(&self, global_offset: [f32; 3]) -> (Vec<[f32; 3]>, Vec<usize>) {
        let faces = self.assemble_faces();

        let to_f32 = |[x, y, z]: [i64; 3]| {
            [
                x as f32,
                y as f32,
                z as f32,
            ]
        };
        let add_offset = |c: [f32; 3]| {
            [
                c[0] + global_offset[0],
                c[1] + global_offset[1],
                c[2] + global_offset[2],
            ]
        };

        let mut vertices = vec![];
        let mut indices = vec![];

        for face in faces {
            let i = vertices.len();
            // Four faces
            vertices.extend(
                face.into_iter()
                    .map(to_f32)
                    .map(add_offset)
            );
            // Two triangles
            indices.extend([i, i+1, i+2, i, i+2, i+3]);
        }

        (vertices, indices)
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

            let ts = chunk.assemble_faces();

            assert_eq!(ts.len(), 0, "there should be zero faces generated");
        }

        #[test]
        fn one() {
            let mut chunk = Chunk::default();
            chunk.get_block_mut([0, 0, 0]).kind = BlockKind::Stone;

            let ts = chunk.assemble_faces();

            assert_eq!(ts.len(), 6, "there should be 6 faces generated");
        }

        #[test]
        fn two() {
            let mut chunk = Chunk::default();
            chunk.get_block_mut([0, 0, 0]).kind = BlockKind::Stone;
            chunk.get_block_mut([0, 0, 1]).kind = BlockKind::Stone;

            let ts = chunk.assemble_faces();

            assert_eq!(ts.len(), 12, "there should be 12 faces generated");
        }
    }
}
