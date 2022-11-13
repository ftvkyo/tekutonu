use cgmath::{InnerSpace, Point3, Vector3};

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
    pub fn get_block(&self, loc: t::PointIntLocal) -> &Block {
        &self.blocks[loc[0]][loc[1]][loc[2]]
    }

    pub fn get_block_mut(&mut self, loc: t::PointIntLocal) -> &mut Block {
        &mut self.blocks[loc[0]][loc[1]][loc[2]]
    }

    pub fn set_block(&mut self, loc: t::PointIntLocal, block: Block) {
        self.blocks[loc[0]][loc[1]][loc[2]] = block;
    }

    fn assemble_faces(&self) -> Vec<[Point3<f32>; 4]> {
        let mut faces = Vec::<[Point3<f32>; 4]>::new();

        for x in 0..c::CHUNK_X_BLOCKS {
            let fx = x as f32;
            for y in 0..c::CHUNK_Y_BLOCKS {
                let fy = y as f32;
                for z in 0..c::CHUNK_Z_BLOCKS {
                    let fz = z as f32;
                    let offset = Vector3::new(fx, fy, fz);
                    let block = self.get_block([x, y, z]);
                    if block.kind == BlockKind::Stone {
                        faces.extend(c::BLOCK_FACES.iter().map(|f| f.map(|p| p + offset)));
                    }
                }
            }
        }

        faces
    }

    pub fn get_render_data(
        &self,
        global_offset: Vector3<f32>,
    ) -> (Vec<Point3<f32>>, Vec<Vector3<f32>>, Vec<usize>) {
        let faces = self.assemble_faces();

        let add_offset = |p: Point3<f32>| p + global_offset;

        // Vertices
        let mut vs = vec![];
        // Normals
        let mut ns = vec![];
        // Indices
        let mut is = vec![];

        for face in faces {
            let i = vs.len();
            // Four vertices
            vs.extend(face.into_iter().map(add_offset));
            ns.extend([
                (face[3] - face[0]).cross(face[1] - face[0]).normalize(), // For vertex 0
                (face[0] - face[1]).cross(face[2] - face[1]).normalize(), // For vertex 1
                (face[1] - face[2]).cross(face[3] - face[2]).normalize(), // For vertex 2
                (face[2] - face[3]).cross(face[0] - face[3]).normalize(), // For vertex 3
            ]);
            // Two triangles
            is.extend([i, i + 1, i + 2, i, i + 2, i + 3]);
        }

        (vs, ns, is)
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
