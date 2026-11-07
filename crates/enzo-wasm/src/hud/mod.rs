//! HUD e overlays em screen space.
//!
//! O HUD tem duas camadas:
//! - uma barra fixa (vida, aura, XP, Mogging) com os dots que abrem submenus;
//! - janelas arrastáveis de status, inventário, equipamento e habilidades.
//!
//! Tudo aqui é leitura do snapshot e estado de UI do cliente. Nenhuma regra de
//! jogo é decidida aqui: cliques só produzem [`HudAction`], que o `app` traduz
//! para comandos do core.
//!
//! O desenho é dividido por responsabilidade:
//! - [`status`] desenha a barra fixa e os toggles;
//! - [`window`] concentra a moldura/arraste das janelas;
//! - `status_window`, `inventory`, `equipment` e `skills` desenham conteúdos;
//! - [`overlays`] desenha banners, dicas e telas de morte/vitória;
//! - as primitivas visuais compartilhadas ficam em `crate::ui`.

mod equipment;
mod inventory;
mod overlays;
mod skills;
mod status;
mod status_window;
mod window;

use enzo_core::input::Attribute;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot, SnapshotKind, WorldSnapshot};
use enzo_core::state::GamePhase;
use macroquad::prelude::*;

use crate::effects::Effects;
use crate::i18n::I18n;
use crate::pointer::{self, Pointer};
use crate::ui;

pub use window::{WindowId, WindowState};

/// Ação de UI que o `app` deve traduzir para um comando do core ou para uma
/// troca de cena.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudAction {
    /// Gasta um ponto no atributo escolhido.
    Allocate(Attribute),
    /// Fecha a tela de level-up e volta ao jogo.
    ConfirmLevelUp,
    /// Reinicia a fase agora (morte/vitória).
    Restart,
    /// Salva e volta ao menu inicial.
    OpenMenu,
}

pub struct Hud {
    windows: [WindowState; WindowId::ALL.len()],
    /// Ordem de empilhamento; o último é o mais acima.
    order: [WindowId; WindowId::ALL.len()],
    focus: Option<WindowId>,
    /// Janela sendo arrastada e o deslocamento entre ponteiro e canto.
    drag: Option<(WindowId, Vec2)>,
    initialized: bool,
    was_level_up: bool,
}

impl Hud {
    pub fn new() -> Self {
        Self {
            windows: [WindowState::new(Vec2::ZERO, Vec2::ZERO); WindowId::ALL.len()],
            order: WindowId::ALL,
            focus: None,
            drag: None,
            initialized: false,
            was_level_up: false,
        }
    }

    /// Desenha o HUD inteiro e devolve no máximo uma ação de UI por frame.
    ///
    /// O clique é edge-triggered, então uma única ação por frame é suficiente.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        snapshot: &WorldSnapshot,
        effects: &Effects,
        i18n: I18n,
        pointer: Pointer,
        time: f32,
        hint_timer: f32,
        save_toast: Option<(&str, f32)>,
    ) -> Option<HudAction> {
        set_default_camera();
        let ui = ui::ui_scale();
        let screen = ui::screen_size();

        if !self.initialized {
            self.init_layout(ui, screen);
            self.initialized = true;
        }
        for state in &mut self.windows {
            state.pos =
                pointer::clamp_window(state.pos, state.size, screen, WindowState::title_height(ui));
        }

        let (entity, player) = find_player(snapshot)?;

        // Level-up abre a janela de status sozinho. O dot continua piscando se
        // o jogador fechar a janela antes de gastar os pontos.
        if snapshot.phase == GamePhase::LevelUp && !self.was_level_up {
            self.open(WindowId::Status);
        }
        self.was_level_up = snapshot.phase == GamePhase::LevelUp;

        let fixed = status::draw_fixed(
            entity,
            player,
            effects,
            i18n,
            pointer,
            time,
            ui,
            &self.windows,
        );
        if let Some(id) = fixed.toggle {
            self.toggle(id);
        }

        let mut action = fixed.menu.then_some(HudAction::OpenMenu);

        overlays::draw_phase_banner(snapshot, i18n, ui);
        overlays::draw_hints(hint_timer, i18n, ui);
        overlays::draw_save_toast(save_toast, i18n, ui);

        if matches!(snapshot.phase, GamePhase::Playing | GamePhase::LevelUp) {
            self.update_drag(pointer, screen, ui);
            self.handle_pointer(pointer, ui);
            if let Some(window_action) = self.draw_windows(pointer, player, i18n, ui) {
                action = Some(window_action);
            }
        }

        draw_flashes(effects, snapshot);

        match snapshot.phase {
            GamePhase::Playing => {}
            GamePhase::LevelUp => overlays::draw_level_up_banner(player, i18n, ui),
            GamePhase::Dead => {
                if let Some(overlay_action) = overlays::draw_dead(snapshot, i18n, pointer, ui) {
                    action = Some(overlay_action);
                }
            }
            GamePhase::Victory => {
                if let Some(overlay_action) = overlays::draw_victory(snapshot, i18n, pointer, ui) {
                    action = Some(overlay_action);
                }
            }
        }

        action
    }

    fn init_layout(&mut self, ui: f32, screen: Vec2) {
        for id in WindowId::ALL {
            let index = id.index();
            let mut state = WindowState::new(default_pos(id) * ui, default_size(id) * ui);
            state.size.x = state.size.x.min(screen.x - 24.0 * ui).max(240.0 * ui);
            state.size.y = state.size.y.min(screen.y - 96.0 * ui).max(180.0 * ui);
            self.windows[index] = state;
        }
    }

    fn update_drag(&mut self, pointer: Pointer, screen: Vec2, ui: f32) {
        let Some((id, offset)) = self.drag else {
            return;
        };
        if !pointer.down {
            self.drag = None;
            return;
        }
        let state = &mut self.windows[id.index()];
        state.pos = pointer::clamp_window(
            pointer.pos + offset,
            state.size,
            screen,
            WindowState::title_height(ui),
        );
    }

    /// Resolve foco, fechamento e início de arraste antes de desenhar. Só a
    /// janela mais acima recebe o clique.
    fn handle_pointer(&mut self, pointer: Pointer, ui: f32) {
        if !pointer.pressed {
            return;
        }
        for index in (0..self.order.len()).rev() {
            let id = self.order[index];
            let state = self.windows[id.index()];
            if !state.open {
                continue;
            }
            if pointer.hovering(state.close_rect(ui)) {
                self.close(id);
                return;
            }
            if pointer.hovering(state.title_rect(ui)) {
                self.focus(id);
                self.drag = Some((id, state.pos - pointer.pos));
                return;
            }
            if pointer.hovering(state.rect()) {
                self.focus(id);
                return;
            }
        }
    }

    fn draw_windows(
        &self,
        pointer: Pointer,
        player: &PlayerSnapshot,
        i18n: I18n,
        ui: f32,
    ) -> Option<HudAction> {
        let interactive = self.top_window_at(pointer.pos);
        let mut action = None;

        // Desenha de baixo para cima para que a janela focada fique por cima.
        for id in self.order.iter().copied() {
            let state = self.windows[id.index()];
            if !state.open {
                continue;
            }

            window::draw_chrome(state, id, i18n, ui, pointer, self.focus == Some(id));

            let active = interactive == Some(id);
            let window_action = match id {
                WindowId::Status => status_window::draw(pointer, state, player, i18n, ui, active),
                WindowId::Inventory => {
                    inventory::draw(state, player, i18n, ui);
                    None
                }
                WindowId::Equipment => {
                    equipment::draw(state, player, i18n, ui);
                    None
                }
                WindowId::Skills => {
                    skills::draw(state, player, i18n, ui);
                    None
                }
            };
            if window_action.is_some() {
                action = window_action;
            }
        }

        action
    }

    fn top_window_at(&self, pos: Vec2) -> Option<WindowId> {
        self.order.iter().rev().copied().find(|id| {
            let state = self.windows[id.index()];
            state.open && pointer::point_in_rect(pos, state.rect())
        })
    }

    fn toggle(&mut self, id: WindowId) {
        if self.windows[id.index()].open {
            self.close(id);
        } else {
            self.open(id);
        }
    }

    fn open(&mut self, id: WindowId) {
        self.windows[id.index()].open = true;
        self.focus(id);
    }

    fn close(&mut self, id: WindowId) {
        self.windows[id.index()].open = false;
        if self.focus == Some(id) {
            self.focus = self
                .order
                .iter()
                .rev()
                .copied()
                .find(|other| *other != id && self.windows[other.index()].open);
        }
        if self.drag.map(|(drag_id, _)| drag_id) == Some(id) {
            self.drag = None;
        }
    }

    fn focus(&mut self, id: WindowId) {
        self.focus = Some(id);
        self.bring_to_front(id);
    }

    fn bring_to_front(&mut self, id: WindowId) {
        let Some(index) = self.order.iter().position(|other| *other == id) else {
            return;
        };
        for current in index..self.order.len() - 1 {
            self.order[current] = self.order[current + 1];
        }
        self.order[self.order.len() - 1] = id;
    }
}

impl Default for Hud {
    fn default() -> Self {
        Self::new()
    }
}

fn default_pos(id: WindowId) -> Vec2 {
    match id {
        WindowId::Status => Vec2::new(430.0, 116.0),
        WindowId::Inventory => Vec2::new(490.0, 150.0),
        WindowId::Equipment => Vec2::new(530.0, 186.0),
        WindowId::Skills => Vec2::new(570.0, 222.0),
    }
}

fn default_size(id: WindowId) -> Vec2 {
    match id {
        WindowId::Status => Vec2::new(360.0, 380.0),
        WindowId::Inventory => Vec2::new(340.0, 300.0),
        WindowId::Equipment => Vec2::new(340.0, 250.0),
        WindowId::Skills => Vec2::new(360.0, 380.0),
    }
}

fn draw_flashes(effects: &Effects, snapshot: &WorldSnapshot) {
    if effects.hurt_flash > 0.0 && snapshot.phase != GamePhase::Dead {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            ui::rgba(255, 60, 50, (effects.hurt_flash * 70.0) as u8),
        );
    }
    if effects.victory_flash > 0.0 {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            ui::rgba(246, 196, 69, (effects.victory_flash * 40.0) as u8),
        );
    }
}

pub(super) fn find_player(snapshot: &WorldSnapshot) -> Option<(&EntitySnapshot, &PlayerSnapshot)> {
    snapshot
        .entities
        .iter()
        .find_map(|entity| match &entity.kind {
            SnapshotKind::Player => entity.player.as_ref().map(|p| (entity, p)),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggling_opens_focuses_and_closes_a_window() {
        let mut hud = Hud::new();
        assert!(!hud.windows[WindowId::Status.index()].open);

        hud.toggle(WindowId::Status);
        assert!(hud.windows[WindowId::Status.index()].open);
        assert_eq!(hud.focus, Some(WindowId::Status));

        hud.toggle(WindowId::Status);
        assert!(!hud.windows[WindowId::Status.index()].open);
        assert_eq!(hud.focus, None);
    }

    #[test]
    fn focus_moves_the_window_to_the_top_of_the_stack() {
        let mut hud = Hud::new();
        hud.focus(WindowId::Inventory);
        assert_eq!(hud.order.last().copied(), Some(WindowId::Inventory));
        hud.focus(WindowId::Skills);
        assert_eq!(hud.order.last().copied(), Some(WindowId::Skills));
    }

    #[test]
    fn top_window_prefers_the_highest_open_window() {
        let mut hud = Hud::new();
        hud.init_layout(1.0, Vec2::new(1280.0, 720.0));
        hud.open(WindowId::Status);
        hud.open(WindowId::Inventory);

        let center = hud.windows[WindowId::Inventory.index()].pos + Vec2::new(10.0, 10.0);
        // As janelas se sobrepõem; a última aberta/focada deve ganhar.
        hud.bring_to_front(WindowId::Inventory);
        assert_eq!(hud.top_window_at(center), Some(WindowId::Inventory));
    }
}
