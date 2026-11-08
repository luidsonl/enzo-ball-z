//! Built-in block definitions.
//!
//! Each block kind lives in its own module (like entity specs), exporting a
//! `DEF: BlockDef`. The registry is indexed by [`BlockId`] discriminant.

pub mod block;
pub mod breakable_stone;
pub mod crate_block;
pub mod empty;
pub mod flag_bottom;
pub mod flag_top;
pub mod ground;
pub mod invisible_block;
pub mod platform;

use super::block::{BlockDef, BlockId};

pub use block::DEF as BLOCK_BRICK;
pub use breakable_stone::DEF as BLOCK_BREAKABLE_STONE;
pub use crate_block::DEF as BLOCK_CRATE;
pub use empty::DEF as BLOCK_EMPTY;
pub use flag_bottom::DEF as BLOCK_FLAG_BOTTOM;
pub use flag_top::DEF as BLOCK_FLAG_TOP;
pub use ground::DEF as BLOCK_GROUND;
pub use invisible_block::DEF as BLOCK_INVISIBLE;
pub use platform::DEF as BLOCK_PLATFORM;

/// All registered block definitions, indexed by [`BlockId`] discriminant order.
pub const BLOCK_DEFS: [BlockDef; 9] = [
    BLOCK_EMPTY,
    BLOCK_GROUND,
    BLOCK_BRICK,
    BLOCK_PLATFORM,
    BLOCK_CRATE,
    BLOCK_FLAG_BOTTOM,
    BLOCK_FLAG_TOP,
    BLOCK_BREAKABLE_STONE,
    BLOCK_INVISIBLE,
];

pub fn block_def(id: BlockId) -> &'static BlockDef {
    &BLOCK_DEFS[id as usize]
}

pub fn block_def_by_symbol(symbol: char) -> Option<&'static BlockDef> {
    BLOCK_DEFS.iter().find(|def| def.symbol == Some(symbol))
}

pub fn block_def_by_id(id: &str) -> Option<&'static BlockDef> {
    let id = id.trim();
    BLOCK_DEFS.iter().find(|def| {
        def.name.eq_ignore_ascii_case(id) || format!("{:?}", def.id).eq_ignore_ascii_case(id)
    })
}

pub fn block_id_by_name(name: &str) -> Option<BlockId> {
    block_def_by_id(name).map(|def| def.id)
}
