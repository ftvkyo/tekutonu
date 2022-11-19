pub type LightLevel = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind {
    Air,
    Solid,
    Light { brightness: LightLevel },
}

impl BlockKind {}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub kind: BlockKind,
}

impl Block {
    pub fn air() -> Self {
        Self {
            kind: BlockKind::Air,
        }
    }

    pub fn solid() -> Self {
        Self {
            kind: BlockKind::Solid,
        }
    }

    pub fn light_source() -> Self {
        Self {
            kind: BlockKind::Light { brightness: 15 },
        }
    }

    pub fn is_transparent(&self) -> bool {
        match self.kind {
            BlockKind::Air => true,
            BlockKind::Solid => false,
            BlockKind::Light { .. } => false,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self::air()
    }
}
