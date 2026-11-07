//! Visual/render metadata for blocks.
//!
//! These are presentation contracts (frame/state identifiers and layout hints).
//! They contain no IO or renderer dependencies.

/// Semantic visual state for a block tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockVisualState {
    pub name: &'static str,
    /// Frame index in the block tilesheet, if rendered.
    pub frame: Option<usize>,
}

/// Visual definition declared by a block module.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockVisual {
    /// Tilesheet asset key/path hint used by clients/tools.
    pub sheet: &'static str,
    pub states: &'static [BlockVisualState],
}

impl BlockVisual {
    pub const fn frame_for_damage(&self, damage: u16, max_hits: Option<u16>) -> Option<usize> {
        let _ = max_hits;
        match self.states {
            [] => None,
            [only] => only.frame,
            states => {
                let idx = damage as usize;
                let idx = if idx >= states.len() {
                    states.len() - 1
                } else {
                    idx
                };
                states[idx].frame
            }
        }
    }
}
