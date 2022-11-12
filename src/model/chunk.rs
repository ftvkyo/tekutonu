use std::collections::HashMap;

use super::{
    block::{Block, BlockKind},
    consts as c,
    types as t,
};

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

    const fn to_index(location: t::PointIntLocal) -> usize {
        let [x, y, z] = location;
        z * c::CHUNK_ONE_SLICE_BLOCKS + y * c::CHUNK_ONE_COLUMN_BLOCKS + x
    }

    const fn to_location(index: usize) -> t::PointIntLocal {
        let x = index % c::CHUNK_ONE_COLUMN_BLOCKS;
        let y = index % c::CHUNK_ONE_SLICE_BLOCKS / c::CHUNK_ONE_COLUMN_BLOCKS;
        let z = index / c::CHUNK_ONE_SLICE_BLOCKS;
        [x, y, z]
    }

    pub fn get_block<'s>(&'s self, location: t::PointIntLocal) -> &'s Block {
        &self.blocks[Self::to_index(location)]
    }

    pub fn get_block_mut(&mut self, location: t::PointIntLocal) -> &mut Block {
        &mut self.blocks[Self::to_index(location)]
    }

    pub fn set_block(&mut self, location: t::PointIntLocal, block: Block) {
        self.blocks[Self::to_index(location)] = block;
    }

    fn assemble_triangles(&self) -> Vec<[t::PointIntGlobal; 3]> {
        let mut triangles = Vec::<[t::PointIntGlobal; 3]>::new();

        for (i, block) in self.blocks.iter().enumerate() {
            if block.kind == BlockKind::Stone {
                let block_offset = Self::to_location(i);
                for triangle in c::BLOCK_TRIANGLES {
                    let triangle = triangle.map(|v| {
                        [
                            v[0] + block_offset[0] as i64,
                            v[1] + block_offset[1] as i64,
                            v[2] + block_offset[2] as i64,
                        ]
                    });
                    triangles.push(triangle);
                }
            }
        }

        triangles
    }

    pub fn get_render_data(&self, global_offset: [f32; 3]) -> (Vec<[f32; 3]>, Vec<usize>) {
        let triangles = self.assemble_triangles();

        let to_f32 = |c| c as f32;

        let mut vertices = vec![];
        let mut indices = vec![];
        let mut vertices_map: HashMap<t::PointIntGlobal, usize> = HashMap::new();

        for triangle in triangles {
            for v in triangle {
                let index = vertices_map.entry(v).or_insert_with(|| {
                    let v = v.map(to_f32);
                    let v = [
                        v[0] + global_offset[0],
                        v[1] + global_offset[1],
                        v[2] + global_offset[2],
                    ];
                    vertices.push(v);
                    vertices.len() - 1
                });
                indices.push(*index);
            }
        }

        (vertices, indices)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const X: usize = c::CHUNK_X_BLOCKS;
    const Y: usize = c::CHUNK_Y_BLOCKS;
    const Z: usize = c::CHUNK_Z_BLOCKS;
    const T: usize = c::CHUNK_TOTAL_BLOCKS;

    mod to_index {
        use super::*;

        #[test]
        fn origin() {
            assert_eq!(
                Chunk::to_index([0, 0, 0]),
                0,
                "indexing should start at 0 for [0, 0, 0]"
            );
        }

        #[test]
        fn last() {
            assert_eq!(
                Chunk::to_index([X - 1, Y - 1, Z - 1]),
                T - 1,
                "max XYZ should yield max index"
            );
        }

        #[test]
        fn growth() {
            assert_eq!(
                Chunk::to_index([1, 0, 0]),
                1,
                "X should be least significant"
            );
            assert_eq!(
                Chunk::to_index([0, 1, 0]),
                X,
                "Y should be the middle significant"
            );
            assert_eq!(
                Chunk::to_index([0, 0, 1]),
                X * Y,
                "Z should be the most significant"
            );
        }

        #[test]
        fn arbitrary_333() {
            assert_eq!(
                Chunk::to_index([3, 3, 3]),
                X * 3 + X * Y * 3 + 3,
                "[3, 3, 3] converts correctly"
            );
        }
    }

    mod to_location {
        use super::*;

        #[test]
        fn origin() {
            assert_eq!(Chunk::to_location(0), [0, 0, 0], "0 should be [0, 0, 0]");
        }

        #[test]
        fn last() {
            assert_eq!(
                Chunk::to_location(T - 1),
                [X - 1, Y - 1, Z - 1],
                "max index should yield max XYZ"
            );
        }

        #[test]
        fn growth() {
            //
            assert_eq!(
                Chunk::to_location(1),
                [1, 0, 0],
                "X should be the least significant"
            );
            assert_eq!(
                Chunk::to_location(X),
                [0, 1, 0],
                "Y should be the middle significant"
            );
            assert_eq!(
                Chunk::to_location(X * Y),
                [0, 0, 1],
                "Z should be the most significant"
            );
        }

        #[test]
        fn parity_with_to_index() {
            for index in 0..T {
                assert_eq!(Chunk::to_index(Chunk::to_location(index)), index);
            }
        }
    }

    mod assemble_triangles {
        use super::*;

        #[test]
        fn empty() {
            let chunk = Chunk::new();

            let ts = chunk.assemble_triangles();

            assert_eq!(ts.len(), 0, "there should be zero triangles generated");
        }

        #[test]
        fn one() {
            let mut chunk = Chunk::new();
            chunk.get_block_mut([0, 0, 0]).kind = BlockKind::Stone;

            let ts = chunk.assemble_triangles();

            assert_eq!(ts.len(), 12, "there should be 12 triangles generated");
        }

        #[test]
        fn two() {
            let mut chunk = Chunk::new();
            chunk.get_block_mut([0, 0, 0]).kind = BlockKind::Stone;
            chunk.get_block_mut([0, 0, 1]).kind = BlockKind::Stone;

            let ts = chunk.assemble_triangles();

            assert_eq!(ts.len(), 24, "there should be 24 triangles generated");
        }
    }
}
