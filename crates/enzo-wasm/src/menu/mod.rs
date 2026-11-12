//! Menu anterior ao jogo.
//!
//! O menu é uma pilha de telas (`MenuScreen`): a tela atual fica no topo e
//! `Back` desempilha. Essa base permite adicionar opções, créditos, seleção de
//! fase e outras telas sem mudar o `app`.
//!
//! Toda a lógica daqui é de UI: navegação, seleção de slot e idioma. O save em
//! si continua sendo `SaveData`, decidido pelo core.

mod main;
mod options;
mod saves;

use enzo_core::progression::{xp_to_next, SaveData};
use macroquad::prelude::*;

use crate::i18n::{I18n, Language, Text};
use crate::pointer::Pointer;
use crate::save;
use crate::ui::{self, rgba, GOLD};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuScreen {
    Main,
    Saves,
    Options,
    Customize,
    Controls,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuAction {
    /// Inicia a sessão no slot escolhido. `save` é `None` para um jogo novo.
    Start { slot: usize, save: Option<SaveData> },
    /// Retoma o jogo que está rodando em segundo plano.
    Resume,
}

/// Eventos internos produzidos por uma tela. O `Menu` aplica depois de desenhar,
/// mantendo a pilha e a persistência em um único lugar.
#[allow(dead_code)]
pub(super) enum MenuEvent {
    Push(MenuScreen),
    Back,
    Start { slot: usize, save: Option<SaveData> },
    RequestDelete(usize),
    Resume,
    SetLanguage(Language),
}

pub struct Menu {
    screens: Vec<MenuScreen>,
    pub(super) slots: [Option<SaveData>; save::SLOT_COUNT],
    pub(super) pending_delete: Option<usize>,
    pub has_background_game: bool,
    pub(super) toast: Option<(String, f32)>,
    time: f32,
    /// Draft character customization used when no saved game is active.
    /// Persisted separately so the character editor works from the main menu.
    pub draft_character: enzo_core::entities::PlayableCharacter,
    pub draft_visual_profile: enzo_core::entities::VisualProfile,
    /// Active key bindings (configurable in Options > Controls).
    pub bindings: crate::input::KeyBindings,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            screens: vec![MenuScreen::Main],
            slots: save::load_slots(),
            pending_delete: None,
            has_background_game: false,
            toast: None,
            time: 0.0,
            draft_character: save::load_draft_character(),
            draft_visual_profile: {
                let c = save::load_draft_character();
                save::load_draft_visual_profile(c)
            },
            bindings: save::load_keybindings(),
        }
    }

    pub fn refresh_slots(&mut self) {
        self.slots = save::load_slots();
    }

    /// Mostra uma mensagem curta no rodapé. Usado pelo `app` quando a fase não
    /// pôde ser criada.
    pub fn notify(&mut self, text: impl Into<String>) {
        self.toast = Some((text.into(), 2.6));
    }

    pub fn current(&self) -> MenuScreen {
        self.screens.last().copied().unwrap_or(MenuScreen::Main)
    }

    pub fn slot(&self, slot: usize) -> Option<&SaveData> {
        self.slots.get(slot).and_then(|s| s.as_ref())
    }

    pub fn pending_delete_slot(&self) -> Option<usize> {
        self.pending_delete
    }

    pub fn toast_text(&self) -> Option<&str> {
        self.toast.as_ref().map(|(text, _)| text.as_str())
    }

    /// Apply a semantic UI action from the DOM bridge. Returns a scene-level action.
    pub fn apply_ui_action(
        &mut self,
        action: crate::ui_bridge::UiAction,
        i18n: &mut I18n,
    ) -> Option<MenuAction> {
        use crate::ui_bridge::UiAction;
        match action {
            UiAction::MenuBack => {
                if self.pending_delete.is_some() {
                    self.pending_delete = None;
                } else {
                    self.back();
                }
                None
            }
            UiAction::MenuOpenSaves => {
                self.push(MenuScreen::Saves);
                None
            }
            UiAction::MenuOpenOptions => {
                self.push(MenuScreen::Options);
                None
            }
            UiAction::MenuOpenCustomize => {
                self.push(MenuScreen::Customize);
                None
            }
            UiAction::MenuOpenControls => {
                self.push(MenuScreen::Controls);
                None
            }
            UiAction::ControlsResetDefaults
            | UiAction::ControlsSetKey { .. }
            | UiAction::ControlsClearAction { .. } => None,
            UiAction::MenuStart { slot, new_game } => {
                let slot = slot as usize;
                if slot >= save::SLOT_COUNT {
                    return None;
                }
                let save = if new_game { None } else { self.slots[slot] };
                Some(MenuAction::Start { slot, save })
            }
            UiAction::MenuResume => Some(MenuAction::Resume),
            UiAction::MenuRequestDelete { slot } => {
                let slot = slot as usize;
                if slot < save::SLOT_COUNT && self.slots[slot].is_some() {
                    self.pending_delete = Some(slot);
                }
                None
            }
            UiAction::MenuCancelDelete => {
                self.pending_delete = None;
                None
            }
            UiAction::MenuConfirmDelete => {
                if let Some(slot) = self.pending_delete.take() {
                    match save::delete_slot(slot) {
                        Ok(()) => {
                            self.refresh_slots();
                            self.toast =
                                Some((format!("{} {}", i18n.t(Text::SlotDeleted), slot + 1), 1.4));
                        }
                        Err(err) => {
                            self.toast =
                                Some((format!("{}: {err}", i18n.t(Text::SaveFailed)), 2.4));
                        }
                    }
                }
                None
            }
            UiAction::SetLanguage { code } => {
                let language = if code == 1 {
                    Language::English
                } else {
                    Language::PtBr
                };
                i18n.set_language(language);
                if let Err(err) = save::store_language(language) {
                    self.toast = Some((format!("{}: {err}", i18n.t(Text::SaveFailed)), 2.4));
                }
                None
            }
            UiAction::SetMobileControls { mode } => {
                let mode = save::MobileControlsMode::from_code(match mode {
                    0 => "off",
                    1 => "on",
                    _ => "auto",
                })
                .unwrap_or_default();
                let _ = save::store_mobile_controls_mode(mode);
                crate::ui_bridge::set_mobile_controls_mode(mode.bridge_code());
                None
            }
            UiAction::SetInterfaceScale { percent } => {
                let scale = (percent as f32 / 100.0).clamp(0.75, 2.0);
                crate::ui::set_interface_scale(scale);
                let _ = save::store_interface_scale(scale);
                None
            }
            UiAction::SetPlayableCharacter { character } => {
                if let Some(c) = enzo_core::entities::PlayableCharacter::from_code(character) {
                    self.draft_character = c;
                    // Exclusive hair/face catalogs per character: reset to sensible defaults.
                    self.draft_visual_profile = enzo_core::entities::VisualProfile::default_for(c);
                    if let Err(err) = save::store_draft_character(c) {
                        self.toast = Some((format!("{}: {err}", i18n.t(Text::SaveFailed)), 2.4));
                    }
                    let _ = save::store_draft_visual_profile(self.draft_visual_profile);
                }
                None
            }
            UiAction::SetVisualProfileField { field, value } => {
                self.draft_visual_profile = self.draft_visual_profile.with_field(field, value);
                if let Err(err) = save::store_draft_visual_profile(self.draft_visual_profile) {
                    self.toast = Some((format!("{}: {err}", i18n.t(Text::SaveFailed)), 2.4));
                }
                None
            }
            _ => None,
        }
    }

    fn push(&mut self, screen: MenuScreen) {
        self.screens.push(screen);
        if screen == MenuScreen::Saves {
            self.refresh_slots();
        }
    }

    fn back(&mut self) {
        if self.screens.len() > 1 {
            self.screens.pop();
        }
    }

    /// Atualiza o menu e devolve a ação que o `app` deve executar.
    /// Tick menu state. Canvas menu UI is disabled; DOM UI renders state and emits actions.
    pub fn frame(
        &mut self,
        _pointer: Pointer,
        _i18n: &mut I18n,
        frame_dt: f32,
    ) -> Option<MenuAction> {
        self.time += frame_dt;
        if let Some((_, remaining)) = self.toast.as_mut() {
            *remaining -= frame_dt;
            if *remaining <= 0.0 {
                self.toast = None;
            }
        }
        // Canvas UI removed: DOM/controller handles menu rendering and actions.
        None
    }

    #[allow(dead_code)]
    fn draw_delete_modal(&mut self, pointer: Pointer, i18n: I18n, ui: f32) {
        let overlay = rgba(4, 4, 12, 190);
        draw_rectangle(0.0, 0.0, screen_width(), screen_height(), overlay);

        let w = (screen_width() - 40.0 * ui).min(460.0 * ui);
        let h = 190.0 * ui;
        let x = (screen_width() - w) * 0.5;
        let y = (screen_height() - h) * 0.5;
        ui::panel(x, y, w, h, rgba(18, 18, 36, 250), rgba(255, 128, 116, 190));

        let slot = self.pending_delete.unwrap_or(0) + 1;
        let title = format!("{} {}", i18n.t(Text::SlotLabel), slot);
        ui::center_text(
            &title,
            x + w * 0.5,
            y + 42.0 * ui,
            (22.0 * ui).max(16.0),
            GOLD,
        );
        ui::center_text(
            i18n.t(Text::DeleteConfirm),
            x + w * 0.5,
            y + 78.0 * ui,
            (16.0 * ui).max(12.0),
            WHITE,
        );

        let button_w = (w - 72.0 * ui) * 0.5;
        let button_h = 46.0 * ui;
        let by = y + h - 66.0 * ui;
        let cancel = Rect::new(x + 24.0 * ui, by, button_w, button_h);
        let confirm = Rect::new(x + 48.0 * ui + button_w, by, button_w, button_h);

        if ui::button(pointer, cancel, i18n.t(Text::Cancel), ui, true, ui::AURA) {
            self.pending_delete = None;
        }
        if ui::button(pointer, confirm, i18n.t(Text::Confirm), ui, true, ui::RED) {
            if let Some(slot) = self.pending_delete.take() {
                match save::delete_slot(slot) {
                    Ok(()) => {
                        self.refresh_slots();
                        self.toast =
                            Some((format!("{} {}", i18n.t(Text::SlotDeleted), slot + 1), 1.4));
                    }
                    Err(err) => {
                        self.toast = Some((format!("{}: {err}", i18n.t(Text::SaveFailed)), 2.4));
                    }
                }
            }
        }
    }
}

impl Default for Menu {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
pub(super) fn slot_summary(i18n: I18n, data: &SaveData) -> String {
    let level = i18n.t(Text::LevelLabel);
    let points = i18n.t(Text::PointsLabel);
    format!(
        "{} {}   ·   {:.0}/{:.0} XP   ·   {} {}",
        level,
        data.level,
        data.xp,
        xp_to_next(data.level),
        data.points,
        points
    )
}

#[allow(dead_code)]
fn draw_menu_background(time: f32, ui: f32) {
    let width = screen_width();
    let height = screen_height();
    clear_background(rgba(10, 12, 28, 255));

    // Gradiente em faixas; o menu não tem câmera nem textura de fundo.
    let bands = 20;
    for index in 0..bands {
        let t = index as f32 / (bands - 1) as f32;
        let color = Color::new(0.07 + 0.10 * t, 0.09 + 0.12 * t, 0.20 + 0.14 * t, 1.0);
        let band_h = height / bands as f32;
        draw_rectangle(0.0, index as f32 * band_h, width, band_h + 1.0, color);
    }

    // Orbes de aura com parallax lento.
    for index in 0..6 {
        let phase = index as f32 * 1.7;
        let x = (width * (0.12 + index as f32 * 0.16) + (time * 0.25 + phase).cos() * 22.0 * ui)
            .rem_euclid(width);
        let y =
            height * (0.18 + 0.12 * (index % 3) as f32) + (time * 0.5 + phase).sin() * 14.0 * ui;
        let radius = (8.0 + index as f32 * 2.0) * ui;
        draw_circle(x, y, radius * 2.6, rgba(94, 242, 255, 10));
        draw_circle(x, y, radius, rgba(94, 242, 255, 38));
    }

    // Linhas de horizonte.
    let horizon = height * 0.72;
    draw_rectangle(0.0, horizon, width, 1.0, rgba(94, 242, 255, 28));
    for index in 0..12 {
        let x = index as f32 * width / 11.0;
        draw_rectangle(x, horizon, 1.0, height - horizon, rgba(94, 242, 255, 10));
    }
}

#[allow(dead_code)]
fn draw_toast(toast: Option<&(String, f32)>, ui: f32) {
    let Some((label, remaining)) = toast else {
        return;
    };
    if *remaining <= 0.0 {
        return;
    }
    let size = (14.0 * ui).max(10.0);
    let dims = measure_text(label, None, size as u16, 1.0);
    let w = dims.width + 28.0 * ui;
    let h = 32.0 * ui;
    let x = (screen_width() - w) * 0.5;
    let y = screen_height() - h - 24.0 * ui;
    draw_rectangle(x, y, w, h, rgba(12, 12, 26, 230));
    draw_rectangle_lines(x, y, w, h, 1.0, rgba(246, 196, 69, 140));
    ui::center_text(
        label,
        x + w * 0.5,
        y + h * 0.5 + size * 0.36,
        size,
        rgba(255, 227, 138, 255),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn menu() -> Menu {
        Menu {
            screens: vec![MenuScreen::Main],
            slots: [None; save::SLOT_COUNT],
            pending_delete: None,
            has_background_game: false,
            toast: None,
            time: 0.0,
            draft_character: Default::default(),
            draft_visual_profile: Default::default(),
            bindings: crate::input::KeyBindings::default(),
        }
    }

    #[test]
    fn screens_form_a_stack() {
        let mut menu = menu();
        assert_eq!(menu.current(), MenuScreen::Main);
        menu.push(MenuScreen::Saves);
        assert_eq!(menu.current(), MenuScreen::Saves);
        menu.push(MenuScreen::Options);
        assert_eq!(menu.current(), MenuScreen::Options);
        menu.back();
        assert_eq!(menu.current(), MenuScreen::Saves);
        menu.back();
        assert_eq!(menu.current(), MenuScreen::Main);
        menu.back();
        assert_eq!(menu.current(), MenuScreen::Main);
    }

    #[test]
    fn slot_summary_uses_the_selected_language() {
        let data = SaveData {
            version: enzo_core::progression::SAVE_VERSION,
            character: Default::default(),
            visual_profile: Default::default(),
            level: 3,
            xp: 20.0,
            points: 2,
            attributes: enzo_core::entities::Attributes::default(),
            loadout: enzo_core::skills::Loadout::default(),
        };
        let pt = slot_summary(I18n::new(Language::PtBr), &data);
        let en = slot_summary(I18n::new(Language::English), &data);
        assert!(pt.contains("Nível"));
        assert!(en.contains("Level"));
        assert!(pt.contains("300"));
    }
}
