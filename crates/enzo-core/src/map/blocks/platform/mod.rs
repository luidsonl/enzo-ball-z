use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::{BlockVisual, BlockVisualState};

pub const DEF: BlockDef = BlockDef {
    id: BlockId::Platform,
    name: "platform",
    symbol: Some('='),
    width: 1,
    height: 1,
    collision: CollisionKind::Platform,
    resistance: Resistance::None,
    special: SpecialKind::None,
    render: RenderKind::Platform,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/blocks/tiles.png",
    states: &[BlockVisualState {
        name: "default",
        frame: Some(3),
    }],
};
