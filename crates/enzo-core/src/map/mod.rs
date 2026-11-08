//! Tilemap, block definitions, and ASCII map parsing.
//!
//! The core does not load files: a level arrives as `&str` and is validated
//! here. Block behavior is data-driven through [`BlockDef`] so new block kinds
//! (water, ladder, breakable blocks, etc.) can be added without scattering tile
//! matches across systems.
//!
//! Organization mirrors the componentized style used by entities: block kinds
//! have their own modules under [`blocks`], while shared contracts and level
//! logic live in focused modules.

pub mod block;
pub mod blocks;
pub mod level;
pub mod parse;
pub mod tile;
pub mod visual;

pub use block::{BlockDef, BlockId, CollisionKind, RenderKind, Resistance, SpecialKind};
pub use blocks::{
    block_def, block_def_by_id, block_def_by_symbol, block_id_by_name, BLOCK_BRICK, BLOCK_CRATE,
    BLOCK_DEFS, BLOCK_EMPTY, BLOCK_GROUND, BLOCK_PLATFORM,
};
pub use level::{Level, TILE_SIZE};
pub use parse::LevelError;
pub use tile::Tile;
pub use visual::{BlockVisual, BlockVisualState};
