//! Loop principal: menu, sessão de jogo, tick fixo, eventos e desenho.
//!
//! O `app` orquestra duas cenas: o menu inicial e uma sessão de jogo. A
//! simulação continua isolada em [`World`]; o cliente acumula tempo de frame e
//! avança o mundo em passos exatos de `1/60 s`. O render interpola entre o
//! snapshot anterior e o atual.

use enzo_core::entities::EntityId;
use enzo_core::events::GameEvent;
use enzo_core::input::{Attribute, PlayerCommand, TickInput};
use enzo_core::levels::LEVEL_1;
use enzo_core::map::LevelError;
use enzo_core::progression::SaveData;
use enzo_core::snapshot::WorldSnapshot;
use enzo_core::world::World;
use enzo_core::DT;
use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::convert::to_vec2;
use crate::effects::Effects;
use crate::hud::{Hud, HudAction};
use crate::i18n::{I18n, Text};
use crate::input::InputState;
use crate::menu::{Menu, MenuAction};
use crate::pointer::Pointer;
use crate::render::{self, Background};
use crate::save;
use crate::sprites::Sprites;
use crate::tiles::TileRenderer;
use crate::ui;
use crate::ui_bridge;

/// Limite de tempo de frame aceito de uma vez. Evita "espiral da morte" quando
/// a aba fica em segundo plano e o navegador entrega um delta enorme.
const MAX_FRAME_TIME: f32 = 0.2;
/// Ticks simulados no máximo por frame. Se o frame atrasou, o resto do tempo é
/// descartado em vez de tentar recuperar tudo de uma vez e travar ainda mais.
const MAX_TICKS_PER_FRAME: u32 = 5;
const HINT_TIME: f32 = 14.0;
const SAVE_TOAST_TIME: f32 = 1.8;

const AURA: Color = Color::new(0.369, 0.949, 1.0, 1.0);
const GOLD: Color = Color::new(0.965, 0.769, 0.271, 1.0);
const RED: Color = Color::new(1.0, 0.42, 0.37, 1.0);
const PURPLE: Color = Color::new(0.722, 0.545, 1.0, 1.0);
const STEEL: Color = Color::new(0.82, 0.86, 0.94, 1.0);

enum Scene {
    Menu(Menu),
    Game(Box<GameSession>),
    MenuOverGame { menu: Menu, game: Box<GameSession> },
}

enum Transition {
    None,
    Start { slot: usize, save: Option<SaveData> },
    OpenMenu,
    Resume,
}

pub struct App {
    scene: Scene,
    sprites: Sprites,
    i18n: I18n,
    pointer: Pointer,
}

impl App {
    pub fn new() -> Self {
        let app = Self {
            scene: Scene::Menu(Menu::new()),
            sprites: Sprites::load(),
            i18n: I18n::new(save::load_language()),
            pointer: Pointer::default(),
        };

        ui::set_interface_scale(save::load_interface_scale());
        ui_bridge::set_mobile_controls_mode(save::load_mobile_controls_mode().bridge_code());

        app
    }

    pub async fn run(&mut self) {
        loop {
            let frame_dt = get_frame_time().clamp(0.0, MAX_FRAME_TIME);
            self.pointer.poll();

            let transition = match &mut self.scene {
                Scene::Menu(menu) => match menu.frame(self.pointer, &mut self.i18n, frame_dt) {
                    Some(MenuAction::Start { slot, save }) => Transition::Start { slot, save },
                    Some(MenuAction::Resume) => Transition::None,
                    None => Transition::None,
                },
                Scene::Game(session) => {
                    if session.frame(frame_dt, self.pointer, &self.sprites) {
                        Transition::OpenMenu
                    } else {
                        Transition::None
                    }
                }
                Scene::MenuOverGame { menu, game } => {
                    // Jogo continua rodando em segundo plano (sem input do HUD).
                    let offscreen = Pointer::at(Vec2::new(-10_000.0, -10_000.0));
                    let _ = game.frame(frame_dt, offscreen, &self.sprites);
                    match menu.frame(self.pointer, &mut self.i18n, frame_dt) {
                        Some(MenuAction::Start { slot, save }) => Transition::Start { slot, save },
                        Some(MenuAction::Resume) => Transition::Resume,
                        None => Transition::None,
                    }
                }
            };

            match transition {
                Transition::None => {}
                Transition::Start { slot, save } => {
                    let tiles = self.sprites.tiles_texture().clone();
                    match GameSession::new(slot, save, tiles, self.i18n) {
                        Ok(session) => self.scene = Scene::Game(Box::new(session)),
                        Err(err) => {
                            if let Scene::Menu(menu) = &mut self.scene {
                                menu.notify(format!(
                                    "{}: {err}",
                                    self.i18n.t(Text::LevelLoadFailed)
                                ));
                            }
                        }
                    }
                }
                Transition::OpenMenu => {
                    if let Scene::Game(mut game) =
                        std::mem::replace(&mut self.scene, Scene::Menu(Menu::new()))
                    {
                        let mut menu = Menu::new();
                        menu.has_background_game = true;
                        game.paused = true;
                        self.scene = Scene::MenuOverGame { menu, game };
                    } else {
                        self.scene = Scene::Menu(Menu::new());
                    }
                }
                Transition::Resume => {
                    if let Scene::MenuOverGame { mut game, .. } =
                        std::mem::replace(&mut self.scene, Scene::Menu(Menu::new()))
                    {
                        game.paused = false;
                        self.scene = Scene::Game(game);
                    }
                }
            }

            next_frame().await;
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

struct GameSession {
    slot: usize,
    world: World,
    tiles: TileRenderer,
    background: Background,
    camera: GameCamera,
    effects: Effects,
    input: InputState,
    hud: Hud,
    i18n: I18n,
    previous: WorldSnapshot,
    current: WorldSnapshot,
    last_command: PlayerCommand,
    accumulator: f32,
    hint_timer: f32,
    save_toast: Option<(String, f32)>,
    time: f32,
    /// Ações de UI acumuladas para o próximo tick.
    pending_allocate: Option<Attribute>,
    pending_confirm: bool,
    pending_restart: bool,
    paused: bool,
}

impl GameSession {
    fn new(
        slot: usize,
        save: Option<SaveData>,
        tiles_texture: Texture2D,
        i18n: I18n,
    ) -> Result<Self, LevelError> {
        let mut world = World::new(LEVEL_1)?;
        if let Some(data) = save {
            world.load_save(&data);
        }

        let tiles = TileRenderer::new(world.level(), tiles_texture);
        let current = world.snapshot();
        let mut camera = GameCamera::new();
        camera.resize(vec2(screen_width(), screen_height()));
        camera.update(0.0, &current, &current, 0.0);

        Ok(Self {
            slot,
            world,
            tiles,
            background: Background::new(),
            camera,
            effects: Effects::new(),
            input: InputState::default(),
            hud: Hud::new(),
            i18n,
            previous: current.clone(),
            current,
            last_command: PlayerCommand::none(),
            accumulator: 0.0,
            hint_timer: HINT_TIME,
            save_toast: None,
            time: 0.0,
            pending_allocate: None,
            pending_confirm: false,
            pending_restart: false,
            paused: false,
        })
    }

    /// Avança e desenha um frame. Retorna `true` quando o jogador pediu para
    /// voltar ao menu.
    fn frame(&mut self, frame_dt: f32, pointer: Pointer, sprites: &Sprites) -> bool {
        self.time += frame_dt;
        self.camera.resize(vec2(screen_width(), screen_height()));
        if !self.paused {
            self.input.poll();
            if self.input.has_activity() || pointer.pressed {
                self.hint_timer = 0.0;
            }
        }
        let mut open_menu = if self.paused {
            false
        } else {
            self.input.take_menu_request()
        };

        if !self.paused {
            self.advance(frame_dt);
        }

        let alpha = (self.accumulator / DT).clamp(0.0, 1.0);
        self.effects.update(frame_dt);
        self.camera
            .update(frame_dt, &self.previous, &self.current, alpha);

        self.hint_timer = (self.hint_timer - frame_dt).max(0.0);
        self.tick_toast(frame_dt);

        render::draw_background(&self.background, &self.camera, self.time);
        render::draw_world(&mut render::RenderContext {
            sprites,
            tiles: &mut self.tiles,
            level: self.world.level(),
            previous: &self.previous,
            current: &self.current,
            alpha,
            camera: &self.camera,
            effects: &self.effects,
            time: self.time,
        });

        let toast = self
            .save_toast
            .as_ref()
            .map(|(text, remaining)| (text.as_str(), *remaining));
        let action = self.hud.draw(
            &self.current,
            &self.effects,
            self.i18n,
            pointer,
            self.time,
            self.hint_timer,
            toast,
        );

        if let Some(action) = action {
            match action {
                HudAction::Allocate(attribute) => self.pending_allocate = Some(attribute),
                HudAction::ConfirmLevelUp => self.pending_confirm = true,
                HudAction::Restart => self.pending_restart = true,
                HudAction::OpenMenu => open_menu = true,
            }
        }

        if open_menu {
            self.persist(Text::SaveReasonProgress);
        }
        open_menu
    }

    /// Consome o tempo acumulado em ticks fixos. O primeiro tick do frame pode
    /// usar edges; os extras só repetem estados contínuos.
    fn advance(&mut self, frame_dt: f32) {
        self.accumulator += frame_dt;

        let ticks = tick_budget(self.accumulator);
        let mut first_tick = true;
        for _ in 0..ticks {
            let command = if first_tick {
                let mut command = self.input.take_command();
                if let Some(attribute) = self.pending_allocate.take() {
                    command.allocate = Some(attribute);
                }
                if self.pending_confirm {
                    command.confirm = true;
                    self.pending_confirm = false;
                }
                if self.pending_restart {
                    command.restart = true;
                    self.pending_restart = false;
                }
                command
            } else {
                self.last_command.without_edges()
            };
            self.last_command = command;
            first_tick = false;

            // Alterna os buffers: `previous` recebe o snapshot antigo e
            // `current` é reescrito no mesmo vetor, sem alocar por tick.
            std::mem::swap(&mut self.previous, &mut self.current);
            let output = self.world.step(TickInput::single(command));
            self.world.write_snapshot(&mut self.current);
            self.handle_events(&output.events);

            self.accumulator -= DT;
        }

        // Frame lento: descarta o backlog restante para não emendar uma fila
        // de ticks e transformar um atraso em travamento.
        if self.accumulator >= DT {
            self.accumulator = 0.0;
        }
    }

    fn handle_events(&mut self, events: &[GameEvent]) {
        use enzo_core::events::SkillRejectReason;
        use enzo_core::skills::SkillId;
        for event in events {
            match *event {
                GameEvent::SkillHit {
                    target,
                    damage,
                    skill,
                    combo,
                    ..
                } => {
                    if let Some(pos) = entity_center(&self.current, target) {
                        let color = match skill {
                            SkillId::Melee => AURA,
                            SkillId::Mogging | SkillId::NutellaShot => PURPLE,
                            _ => STEEL,
                        };
                        self.effects.damage(pos, damage, color);
                        self.effects
                            .sparks(pos, color, if combo >= 3 { 10 } else { 6 });
                    }
                    self.camera.add_shake(if matches!(skill, SkillId::Mogging) {
                        4.0
                    } else {
                        2.0
                    });
                }
                GameEvent::SkillProjectileSpawned { owner, skill, .. } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        let label = self.i18n.skill(skill);
                        self.effects.sparks(pos, PURPLE, 10);
                        self.effects.text(pos, label, PURPLE, 12.0);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::SkillProjectileBlocked { owner, .. } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        self.effects
                            .text(pos, self.i18n.t(Text::EffectBlocked), STEEL, 10.0);
                        self.effects.sparks(pos, STEEL, 5);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::SkillRejected { player, reason, .. } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        match reason {
                            SkillRejectReason::NoAura => {
                                self.effects.no_aura(pos, self.i18n.t(Text::EffectNoAura));
                            }
                            SkillRejectReason::Cooldown => {
                                self.effects.text(
                                    pos,
                                    self.i18n.t(Text::SkillCooldown),
                                    STEEL,
                                    10.0,
                                );
                            }
                            SkillRejectReason::Unavailable(_) => {
                                self.effects
                                    .text(pos, self.i18n.t(Text::SkillLocked), STEEL, 10.0);
                            }
                            SkillRejectReason::EmptySlot => {}
                        }
                    }
                    self.camera.add_shake(1.0);
                }
                GameEvent::SkillActivated { .. } | GameEvent::LoadoutChanged { .. } => {}
                GameEvent::PlayerDamaged { player, damage, .. } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.hurt(pos);
                        self.effects.damage(pos, damage, RED);
                    }
                    self.camera.add_shake(5.0);
                }
                GameEvent::EnemyKilled { enemy, xp } => {
                    if let Some(pos) = entity_center(&self.current, enemy) {
                        self.effects.xp(pos, xp);
                        self.effects.sparks(pos, GOLD, 14);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::LevelUp { player, level } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(
                            pos,
                            format!("{} {level}!", self.i18n.t(Text::EffectLevel)),
                            GOLD,
                            16.0,
                        );
                        self.effects.sparks(pos, GOLD, 18);
                    }
                    self.effects.aura_flash = 0.7;
                    self.camera.add_shake(2.0);
                }
                GameEvent::PlayerDied { player } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects
                            .text(pos, self.i18n.t(Text::EffectDeath), RED, 18.0);
                        self.effects.sparks(pos, RED, 16);
                    }
                    self.effects.hurt_flash = 0.7;
                    self.camera.add_shake(9.0);
                    self.persist(Text::SaveReasonDeath);
                }
                GameEvent::Victory { player } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects
                            .text(pos, self.i18n.t(Text::EffectVictory), GOLD, 20.0);
                        self.effects.sparks(pos, GOLD, 24);
                    }
                    self.effects.victory_flash = 1.2;
                    self.camera.add_shake(4.0);
                }
                GameEvent::AttributeAllocated { player, attribute } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(
                            pos,
                            format!("+{}", self.i18n.attribute(attribute)),
                            AURA,
                            11.0,
                        );
                    }
                    self.persist(Text::SaveReasonAttribute);
                }
                GameEvent::PhaseRestarted { .. } => {
                    self.camera.reset();
                    self.effects.clear();
                }
                GameEvent::SaveRequested => self.persist(Text::SaveReasonProgress),
            }
        }
    }

    fn persist(&mut self, reason: Text) {
        let Some(data) = self.world.save_data() else {
            return;
        };
        let reason = self.i18n.t(reason).to_owned();
        match save::store_slot(self.slot, &data) {
            Ok(()) => self.save_toast = Some((reason, SAVE_TOAST_TIME)),
            Err(err) => {
                warn!("save: {err}");
                self.save_toast = Some((
                    format!("{}: {err}", self.i18n.t(Text::SaveFailed)),
                    SAVE_TOAST_TIME,
                ));
            }
        }
    }

    fn tick_toast(&mut self, frame_dt: f32) {
        let expired = match self.save_toast.as_mut() {
            Some((_, remaining)) => {
                *remaining -= frame_dt;
                *remaining <= 0.0
            }
            None => false,
        };
        if expired {
            self.save_toast = None;
        }
    }
}

/// Quantos ticks de simulação cabem no tempo acumulado, respeitando o teto por
/// frame. Fora do laço para poder testar sem navegador.
fn tick_budget(accumulator: f32) -> u32 {
    if accumulator < DT {
        return 0;
    }
    (accumulator / DT).floor().min(MAX_TICKS_PER_FRAME as f32) as u32
}

/// Centro visual de uma entidade no snapshot atual.
fn entity_center(snapshot: &WorldSnapshot, id: EntityId) -> Option<Vec2> {
    snapshot
        .entities
        .iter()
        .find(|entity| entity.id == id)
        .map(|entity| to_vec2(entity.pos + entity.size * 0.5))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_budget_waits_for_a_full_step() {
        assert_eq!(tick_budget(0.0), 0);
        assert_eq!(tick_budget(DT * 0.99), 0);
        assert_eq!(tick_budget(DT), 1);
    }

    #[test]
    fn tick_budget_caps_catch_up_ticks() {
        assert_eq!(tick_budget(DT * 3.0), 3);
        assert_eq!(tick_budget(DT * 12.0), MAX_TICKS_PER_FRAME);
        assert_eq!(tick_budget(MAX_FRAME_TIME), MAX_TICKS_PER_FRAME);
    }
}
