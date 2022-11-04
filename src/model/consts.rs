const CHUNK_EDGE_BLOCKS: usize = 16;
pub const CHUNK_X_BLOCKS: usize = CHUNK_EDGE_BLOCKS;
pub const CHUNK_Y_BLOCKS: usize = CHUNK_EDGE_BLOCKS;
pub const CHUNK_Z_BLOCKS: usize = CHUNK_EDGE_BLOCKS;

pub const CHUNK_ONE_COLUMN_BLOCKS: usize = CHUNK_Y_BLOCKS;
pub const CHUNK_ONE_SLICE_BLOCKS: usize = CHUNK_Y_BLOCKS * CHUNK_Z_BLOCKS;
pub const CHUNK_TOTAL_BLOCKS: usize = CHUNK_X_BLOCKS * CHUNK_Y_BLOCKS * CHUNK_Z_BLOCKS;

const REGION_SIDE_CHUNKS: usize = 16;
const REGION_HEIGHT_CHUNKS: usize = 16;
pub const REGION_X_CHUNKS: usize = REGION_SIDE_CHUNKS;
pub const REGION_Y_CHUNKS: usize = REGION_HEIGHT_CHUNKS;
pub const REGION_Z_CHUNKS: usize = REGION_SIDE_CHUNKS;

pub const REGION_ONE_COLUMN_CHUNKS: usize = REGION_Y_CHUNKS;
pub const REGION_ONE_SLICE_CHUNKS: usize = REGION_Y_CHUNKS * REGION_Z_CHUNKS;
pub const REGION_TOTAL_CHUNKS: usize = REGION_X_CHUNKS * REGION_Y_CHUNKS * REGION_Z_CHUNKS;

const BLOCK_POINTS: [super::types::PointIntGlobal; 8] = [
    [0, 0, 0],
    [0, 0, 1],
    [0, 1, 0],
    [0, 1, 1],
    [1, 0, 0],
    [1, 0, 1],
    [1, 1, 0],
    [1, 1, 1],
];

// Triangles of a cube, all clockwise if looking from outsid eof the cube
pub const BLOCK_TRIANGLES: [[super::types::PointIntGlobal; 3]; 12] = {
    let [a, b, c, d, e, f, g, h] = BLOCK_POINTS;
    [
        [a, c, g], // Front top
        [a, g, e], // Front bottom
        [b, d, c], // Left top
        [b, c, a], // Left bottom
        [f, h, d], // Back top
        [f, d, b], // Back bottom
        [e, g, h], // Right top
        [e, h, f], // Right bottom
        [c, d, h], // Top left
        [c, h, g], // Top right
        [e, b, a], // Bottom left
        [e, f, b], // Bottom right
    ]
};
