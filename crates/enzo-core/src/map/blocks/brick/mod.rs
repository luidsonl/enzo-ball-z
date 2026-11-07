use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::{BlockVisual, BlockVisualState};

pub const DEF: BlockDef = BlockDef {
    id: BlockId::Block,
    name: "block",
    symbol: Some('B'),
    collision: CollisionKind::Solid,
    resistance: Resistance::Unbreakable,
    special: SpecialKind::None,
    render: RenderKind::Block,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/sprites/tiles.png",
    states: &[BlockVisualState {
        name: "default",
        frame: Some(2),
    }],
};
