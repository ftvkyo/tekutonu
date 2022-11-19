use std::ops::Add;

use super::consts as c;


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointIntLocal(pub [isize; 3]);

impl PointIntLocal {
    pub const fn new(x: isize, y: isize, z: isize) -> Self {
        Self([x, y, z])
    }

    pub fn x(&self) -> isize {
        self.0[0]
    }

    pub fn y(&self) -> isize {
        self.0[1]
    }

    pub fn z(&self) -> isize {
        self.0[2]
    }

    pub fn ux(&self) -> usize {
        self.0[0].try_into().unwrap()
    }

    pub fn uy(&self) -> usize {
        self.0[1].try_into().unwrap()
    }

    pub fn uz(&self) -> usize {
        self.0[2].try_into().unwrap()
    }

    pub fn is_on_chunk_face(&self) -> bool {
        self.x() == 0
            || self.x() == c::CHUNK_X_BLOCKS as isize - 1
            || self.y() == 0
            || self.y() == c::CHUNK_Y_BLOCKS as isize - 1
            || self.z() == 0
            || self.z() == c::CHUNK_Z_BLOCKS as isize - 1
    }

    pub fn is_in_chunk(&self) -> bool {
        self.x() >= 0
            && self.x() < c::CHUNK_X_BLOCKS as isize
            && self.y() >= 0
            && self.y() < c::CHUNK_Y_BLOCKS as isize
            && self.z() >= 0
            && self.z() < c::CHUNK_Z_BLOCKS as isize
    }
}

impl From<[usize; 3]> for PointIntLocal {
    fn from([x, y, z]: [usize; 3]) -> Self {
        Self([x as isize, y as isize, z as isize])
    }
}

impl From<[isize; 3]> for PointIntLocal {
    fn from(arr: [isize; 3]) -> Self {
        Self(arr)
    }
}

impl AsRef<[isize; 3]> for PointIntLocal {
    fn as_ref(&self) -> &[isize; 3] {
        &self.0
    }
}


impl Add<&PointIntLocal> for PointIntLocal {
    type Output = PointIntLocal;

    fn add(self, rhs: &PointIntLocal) -> Self::Output {
        Self::Output::new(self.x() + rhs.x(), self.y() + rhs.y(), self.z() + rhs.z())
    }
}

impl std::fmt::Display for PointIntLocal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(local {}, {}, {})", self.x(), self.y(), self.z())
    }
}

pub struct PointGlobal(pub [f32; 3]);
