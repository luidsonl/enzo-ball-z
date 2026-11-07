use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::BlockVisual;

pub const DEF: BlockDef = BlockDef {
    id: BlockId::Empty,
    name: "empty",
    symbol: Some('.'),
    collision: CollisionKind::Passable,
    resistance: Resistance::None,
    special: SpecialKind::None,
    render: RenderKind::Empty,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/sprites/tiles.png",
    states: &[],
};
