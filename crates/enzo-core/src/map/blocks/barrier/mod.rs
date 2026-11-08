use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::BlockVisual;

pub const DEF: BlockDef = BlockDef {
    id: BlockId::Barrier,
    name: "barrier",
    symbol: Some('X'),
    width: 1,
    height: 1,
    collision: CollisionKind::Solid,
    resistance: Resistance::Unbreakable,
    special: SpecialKind::None,
    render: RenderKind::Empty,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/blocks/tiles.png",
    states: &[],
};
