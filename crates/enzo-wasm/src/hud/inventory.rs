#![allow(dead_code)]
//! Inventory UI removed: items are collected directly (auto/touch or interact) and
//! equipment is placed directly into character slots.

use enzo_core::snapshot::PlayerSnapshot;

use crate::i18n::I18n;

use super::window::WindowState;

pub(super) fn draw(_state: WindowState, _player: &PlayerSnapshot, _i18n: I18n, _ui: f32) {
    // Inventory is no longer used.
}
