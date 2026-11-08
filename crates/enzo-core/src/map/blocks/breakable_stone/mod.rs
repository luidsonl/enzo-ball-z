use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::{BlockVisual, BlockVisualState};

pub const DEF: BlockDef = BlockDef {
    id: BlockId::BreakableStone,
    name: "breakable_stone",
    symbol: Some('S'),
    width: 1,
    height: 1,
    collision: CollisionKind::Solid,
    resistance: Resistance::Hits(3),
    special: SpecialKind::Destructible,
    render: RenderKind::BreakableStone,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/blocks/tiles.png",
    states: &[
        BlockVisualState {
            name: "intact",
            frame: Some(9),
        },
        BlockVisualState {
            name: "damaged_1",
            frame: Some(10),
        },
        BlockVisualState {
            name: "damaged_2",
            frame: Some(11),
        },
        BlockVisualState {
            name: "broken",
            frame: None,
        },
    ],
};
