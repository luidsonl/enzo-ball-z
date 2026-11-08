use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::{BlockVisual, BlockVisualState};

pub const DEF: BlockDef = BlockDef {
    id: BlockId::Ground,
    name: "ground",
    symbol: Some('#'),
    width: 1,
    height: 1,
    collision: CollisionKind::Solid,
    resistance: Resistance::Unbreakable,
    special: SpecialKind::None,
    render: RenderKind::Ground,
};

/// Ground visual is context-sensitive (top/fill); frames are resolved by the
/// renderer using adjacency and the tilesheet contract.
pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/blocks/tiles.png",
    states: &[
        BlockVisualState {
            name: "top",
            frame: Some(0),
        },
        BlockVisualState {
            name: "fill",
            frame: Some(1),
        },
    ],
};
