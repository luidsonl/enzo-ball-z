//! Block type definitions and properties.

/// Stable block identifier stored in the tilemap.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BlockId {
    Empty = 0,
    Ground = 1,
    Block = 2,
    Platform = 3,
    Crate = 4,
    /// Lower part of the 2-tile goal flag marker.
    FlagBottom = 5,
    /// Upper part of the 2-tile goal flag marker.
    FlagTop = 6,
    /// Solid stone block that can be damaged and destroyed.
    BreakableStone = 7,
    /// Invisible, impassable boundary block that prevents leaving the level.
    InvisibleBlock = 8,
}

/// How a block participates in movement/collision.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollisionKind {
    /// No collision.
    Passable,
    /// Blocks horizontal movement and acts as floor/ceiling.
    Solid,
    /// Standable from above while falling; passable from below/sides.
    Platform,
    /// Passable space with fluid behavior (swimming rules handled by movement).
    Fluid,
    /// Passable space that can be climbed (climbing rules handled by movement).
    Climbable,
    /// Passable trigger volume (goal/special interactions).
    Trigger,
}

/// Optional durability for blocks that can be damaged/destroyed later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resistance {
    None,
    /// Hit points required to break the block.
    Hits(u16),
    Unbreakable,
}

/// Special block behavior hook. Gameplay resolution stays in core systems.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpecialKind {
    None,
    Swim,
    Climb,
    GoalTrigger,
    Destructible,
}

/// Visual hint for the default tile renderer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderKind {
    Empty,
    /// Uses the "solid above => fill, else top" rule from current behavior.
    Ground,
    Block,
    Platform,
    Crate,
    Water,
    Ladder,
    FlagBottom,
    FlagTop,
    BreakableStone,
}

/// Static definition for a block kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockDef {
    pub id: BlockId,
    pub name: &'static str,
    /// ASCII symbol used by the map parser to place this block directly (1x1 blocks).
    pub symbol: Option<char>,
    /// Footprint width in tiles.
    pub width: u8,
    /// Footprint height in tiles.
    pub height: u8,
    pub collision: CollisionKind,
    pub resistance: Resistance,
    pub special: SpecialKind,
    pub render: RenderKind,
}

impl BlockDef {
    pub const fn is_solid(self) -> bool {
        matches!(self.collision, CollisionKind::Solid)
    }

    pub const fn is_platform(self) -> bool {
        matches!(self.collision, CollisionKind::Platform)
    }

    pub const fn is_standable(self) -> bool {
        self.is_solid() || self.is_platform()
    }

    pub const fn is_fluid(self) -> bool {
        matches!(self.collision, CollisionKind::Fluid)
    }

    pub const fn is_climbable(self) -> bool {
        matches!(self.collision, CollisionKind::Climbable)
    }

    pub const fn max_hits(self) -> Option<u16> {
        match self.resistance {
            Resistance::Hits(hits) => Some(hits),
            Resistance::None | Resistance::Unbreakable => None,
        }
    }
}
