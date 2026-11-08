use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::{BlockVisual, BlockVisualState};

pub const DEF: BlockDef = BlockDef {
    id: BlockId::Crate,
    name: "crate",
    symbol: Some('D'),
    width: 1,
    height: 1,
    collision: CollisionKind::Solid,
    resistance: Resistance::Hits(3),
    special: SpecialKind::Destructible,
    render: RenderKind::Crate,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/blocks/tiles.png",
    states: &[
        BlockVisualState {
            name: "intact",
            frame: Some(4),
        },
        BlockVisualState {
            name: "damaged_1",
            frame: Some(5),
        },
        BlockVisualState {
            name: "damaged_2",
            frame: Some(6),
        },
        BlockVisualState {
            name: "broken",
            frame: None,
        },
    ],
};
