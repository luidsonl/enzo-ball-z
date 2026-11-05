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

use main::draw as draw_main;
use options::draw as draw_options;
use saves::draw as draw_saves;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuScreen {
    Main,
    Saves,
    Options,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MenuAction {
    /// Inicia a sessão no slot escolhido. `save` é `None` para um jogo novo.
    Start { slot: usize, save: Option<SaveData> },
}

/// Eventos internos produzidos por uma tela. O `Menu` aplica depois de desenhar,
/// mantendo a pilha e a persistência em um único lugar.
pub(super) enum MenuEvent {
    Push(MenuScreen),
    Back,
    Start { slot: usize, save: Option<SaveData> },
    RequestDelete(usize),
    SetLanguage(Language),
}

pub struct Menu {
    screens: Vec<MenuScreen>,
    pub(super) slots: [Option<SaveData>; save::SLOT_COUNT],
    pub(super) pending_delete: Option<usize>,
    pub(super) toast: Option<(String, f32)>,
    time: f32,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            screens: vec![MenuScreen::Main],
            slots: save::load_slots(),
            pending_delete: None,
            toast: None,
            time: 0.0,
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

    pub(super) fn current(&self) -> MenuScreen {
        self.screens.last().copied().unwrap_or(MenuScreen::Main)
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
    pub fn frame(
        &mut self,
        pointer: Pointer,
        i18n: &mut I18n,
        frame_dt: f32,
    ) -> Option<MenuAction> {
        self.time += frame_dt;
        if let Some((_, remaining)) = self.toast.as_mut() {
            *remaining -= frame_dt;
            if *remaining <= 0.0 {
                self.toast = None;
            }
        }

        let ui = ui::ui_scale();
        draw_menu_background(self.time, ui);

        let event = if self.pending_delete.is_some() {
            None
        } else {
            match self.current() {
                MenuScreen::Main => draw_main(pointer, *i18n, ui, self.time),
                MenuScreen::Saves => draw_saves(pointer, *i18n, ui, self.time, self),
                MenuScreen::Options => draw_options(pointer, i18n, ui, self.time),
            }
        };

        let mut action = None;
        if self.pending_delete.is_some() {
            // O modal bloqueia a tela de baixo: cliques fora dele não podem
            // navegar nem iniciar uma partida enquanto a confirmação está aberta.
            self.draw_delete_modal(pointer, *i18n, ui);
        } else if let Some(event) = event {
            match event {
                MenuEvent::Push(screen) => self.push(screen),
                MenuEvent::Back => self.back(),
                MenuEvent::Start { slot, save } => action = Some(MenuAction::Start { slot, save }),
                MenuEvent::RequestDelete(slot) => self.pending_delete = Some(slot),
                MenuEvent::SetLanguage(language) => {
                    i18n.set_language(language);
                    if let Err(err) = save::store_language(language) {
                        self.toast = Some((format!("{}: {err}", i18n.t(Text::SaveFailed)), 2.4));
                    }
                }
            }
        }

        draw_toast(self.toast.as_ref(), ui);

        action
    }

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
            macroquad::logging::info!("delete modal: confirm clicked");
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
            toast: None,
            time: 0.0,
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
            level: 3,
            xp: 20.0,
            points: 2,
            attributes: enzo_core::entities::Attributes::default(),
        };
        let pt = slot_summary(I18n::new(Language::PtBr), &data);
        let en = slot_summary(I18n::new(Language::English), &data);
        assert!(pt.contains("Nível"));
        assert!(en.contains("Level"));
        assert!(pt.contains("300"));
    }
}
