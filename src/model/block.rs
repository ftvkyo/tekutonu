#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind {
    Air,
    Stone,
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub kind: BlockKind,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            kind: BlockKind::Air,
        }
    }
}
