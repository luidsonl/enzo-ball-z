use crate::map::block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
use crate::map::visual::{BlockVisual, BlockVisualState};

pub const DEF: BlockDef = BlockDef {
    id: BlockId::FlagBottom,
    name: "flag_bottom",
    symbol: None, // placed by `F` marker parser (2-tile goal flag)
    width: 1,
    height: 1,
    collision: CollisionKind::Passable,
    resistance: Resistance::None,
    special: SpecialKind::GoalTrigger,
    render: RenderKind::FlagBottom,
};

pub const VISUAL: BlockVisual = BlockVisual {
    sheet: "assets/blocks/tiles.png",
    states: &[BlockVisualState {
        name: "default",
        frame: Some(7),
    }],
};
