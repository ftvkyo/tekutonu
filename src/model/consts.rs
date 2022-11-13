use cgmath::Point3;

const CHUNK_EDGE_BLOCKS: usize = 16;

pub const CHUNK_X_BLOCKS: usize = CHUNK_EDGE_BLOCKS;
pub const CHUNK_Y_BLOCKS: usize = CHUNK_EDGE_BLOCKS;
pub const CHUNK_Z_BLOCKS: usize = CHUNK_EDGE_BLOCKS;

pub const CHUNK_TOTAL_BLOCKS: usize = CHUNK_X_BLOCKS * CHUNK_Y_BLOCKS * CHUNK_Z_BLOCKS;

const REGION_SIDE_CHUNKS: usize = 16;
const REGION_HEIGHT_CHUNKS: usize = 16;

pub const REGION_X_CHUNKS: usize = REGION_SIDE_CHUNKS;
pub const REGION_Y_CHUNKS: usize = REGION_HEIGHT_CHUNKS;
pub const REGION_Z_CHUNKS: usize = REGION_SIDE_CHUNKS;

pub const REGION_TOTAL_CHUNKS: usize = REGION_X_CHUNKS * REGION_Y_CHUNKS * REGION_Z_CHUNKS;

pub const BLOCK_FACES: [[Point3<f32>; 4]; 6] = {
    // l, r - left, right
    // b, t - bottom, top
    // n, f - near, far

    let [l, b, n] = [0.0, 0.0, 0.0];
    let [r, t, f] = [1.0, 1.0, 1.0];

    // Binary counting
    let lbn = Point3::new(l, b, n);
    let lbf = Point3::new(l, b, f);
    let ltn = Point3::new(l, t, n);
    let ltf = Point3::new(l, t, f);
    let rbn = Point3::new(r, b, n);
    let rbf = Point3::new(r, b, f);
    let rtn = Point3::new(r, t, n);
    let rtf = Point3::new(r, t, f);

    // Encode block faces:
    // front, left, back, right, bottom, top
    [
        [lbn, rbn, rtn, ltn],
        [lbf, lbn, ltn, ltf],
        [rbf, lbf, ltf, rtf],
        [rbn, rbf, rtf, rtn],
        [lbf, rbf, rbn, lbn],
        [ltn, rtn, rtf, ltf],
    ]
};
