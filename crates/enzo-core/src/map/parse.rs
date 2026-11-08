//! ASCII map parsing and symbol actions.

use std::fmt;

use super::block::BlockId;
use super::blocks;

/// Map symbol semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SymbolAction {
    Place(BlockId),
    /// Marker: records a spawn point and leaves an empty tile.
    PlayerSpawn,
    /// Marker: records an enemy spawn point and leaves an empty tile.
    EnemySpawn,
    /// Marker: places a 2-tile goal flag (bottom+top blocks) and records the trigger.
    Flag,
}

pub fn symbol_action(symbol: char) -> Option<SymbolAction> {
    if let Some(def) = blocks::block_def_by_symbol(symbol) {
        return Some(SymbolAction::Place(def.id));
    }
    match symbol {
        'P' => Some(SymbolAction::PlayerSpawn),
        'E' => Some(SymbolAction::EnemySpawn),
        'F' => Some(SymbolAction::Flag),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LevelError {
    Empty,
    InconsistentWidth {
        row: usize,
        expected: usize,
        found: usize,
    },
    UnknownTile {
        row: usize,
        col: usize,
        symbol: char,
    },
    NoSpawn,
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty map"),
            Self::InconsistentWidth {
                row,
                expected,
                found,
            } => write!(f, "row {row} has {found} columns; expected {expected}"),
            Self::UnknownTile { row, col, symbol } => {
                write!(f, "unknown symbol '{symbol}' at ({row}, {col})")
            }
            Self::NoSpawn => write!(f, "map has no player spawn 'P'"),
        }
    }
}

impl std::error::Error for LevelError {}
