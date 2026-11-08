//! Tile cell representation.

use super::block::{BlockId, RenderKind};
use super::blocks;

/// A tile cell in the level grid (effective/base representation helper).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Tile(BlockId);

impl Tile {
    pub const EMPTY: Self = Self(BlockId::Empty);
    pub const GROUND: Self = Self(BlockId::Ground);
    pub const BLOCK: Self = Self(BlockId::Block);
    pub const PLATFORM: Self = Self(BlockId::Platform);
    pub const CRATE: Self = Self(BlockId::Crate);
    pub const FLAG_BOTTOM: Self = Self(BlockId::FlagBottom);
    pub const FLAG_TOP: Self = Self(BlockId::FlagTop);
    pub const BREAKABLE_STONE: Self = Self(BlockId::BreakableStone);

    pub const fn new(id: BlockId) -> Self {
        Self(id)
    }

    pub const fn kind(self) -> BlockId {
        self.0
    }

    pub fn def(self) -> &'static super::block::BlockDef {
        blocks::block_def(self.0)
    }

    pub fn is_solid(self) -> bool {
        self.def().is_solid()
    }

    pub fn is_platform(self) -> bool {
        self.def().is_platform()
    }

    pub fn is_standable(self) -> bool {
        self.def().is_standable()
    }

    pub fn is_fluid(self) -> bool {
        self.def().is_fluid()
    }

    pub fn is_climbable(self) -> bool {
        self.def().is_climbable()
    }

    pub fn render_kind(self) -> RenderKind {
        self.def().render
    }
}

impl From<BlockId> for Tile {
    fn from(value: BlockId) -> Self {
        Self(value)
    }
}
