pub type LightLevel = u8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BlockKind {
    Air {
        light_local: LightLevel,
        light_global: LightLevel,
    },
    Solid,
    Light {
        brightness: LightLevel,
    },
}

impl BlockKind {
    pub fn air() -> Self {
        Self::Air {
            light_local: 0,
            light_global: 0,
        }
    }

    pub fn solid() -> Self {
        Self::Solid
    }

    pub fn light() -> Self {
        Self::Light {
            brightness: 15,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub kind: BlockKind,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            kind: BlockKind::air(),
        }
    }
}
