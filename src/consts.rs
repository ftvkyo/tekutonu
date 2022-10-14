const CHUNK_EDGE_BLOCKS: usize = 32;
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