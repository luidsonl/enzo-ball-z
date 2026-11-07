//! Built-in block definitions.
//!
//! Each block kind lives in its own module (like entity specs), exporting a
//! `DEF: BlockDef`. The registry is indexed by [`BlockId`] discriminant.

pub mod block;
pub mod crate_block;
pub mod empty;
pub mod ground;
pub mod platform;

use super::block::{BlockDef, BlockId};

pub use block::DEF as BLOCK_BRICK;
pub use crate_block::DEF as BLOCK_CRATE;
pub use empty::DEF as BLOCK_EMPTY;
pub use ground::DEF as BLOCK_GROUND;
pub use platform::DEF as BLOCK_PLATFORM;

/// All registered block definitions, indexed by [`BlockId`] discriminant order.
pub const BLOCK_DEFS: [BlockDef; 5] = [
    BLOCK_EMPTY,
    BLOCK_GROUND,
    BLOCK_BRICK,
    BLOCK_PLATFORM,
    BLOCK_CRATE,
];

pub fn block_def(id: BlockId) -> &'static BlockDef {
    &BLOCK_DEFS[id as usize]
}

pub fn block_def_by_symbol(symbol: char) -> Option<&'static BlockDef> {
    BLOCK_DEFS.iter().find(|def| def.symbol == Some(symbol))
}
