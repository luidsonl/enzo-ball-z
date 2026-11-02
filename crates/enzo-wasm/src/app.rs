//! Loop principal: tick fixo, tradução de input, eventos e desenho.
//!
//! O cliente acumula tempo de frame e avança o [`World`] em passos exatos de
//! `1/60 s`. O render interpola entre o snapshot anterior e o atual; a
//! simulação nunca depende do framerate.

use enzo_core::entities::EntityId;
use enzo_core::events::GameEvent;
use enzo_core::input::{PlayerCommand, TickInput};
use enzo_core::levels::LEVEL_1;
use enzo_core::map::LevelError;
use enzo_core::snapshot::WorldSnapshot;
use enzo_core::world::World;
use enzo_core::DT;
use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::convert::to_vec2;
use crate::effects::Effects;
use crate::hud;
use crate::input::InputState;
use crate::render;
use crate::save;
use crate::sprites::Sprites;

/// Limite de tempo de frame aceito de uma vez. Evita "espiral da morte" quando
/// a aba fica em segundo plano e o navegador entrega um delta enorme.
const MAX_FRAME_TIME: f32 = 0.2;
const HINT_TIME: f32 = 14.0;
const SAVE_TOAST_TIME: f32 = 1.8;

const AURA: Color = Color::new(0.369, 0.949, 1.0, 1.0);
const GOLD: Color = Color::new(0.965, 0.769, 0.271, 1.0);
const RED: Color = Color::new(1.0, 0.42, 0.37, 1.0);
const PURPLE: Color = Color::new(0.722, 0.545, 1.0, 1.0);
const STEEL: Color = Color::new(0.82, 0.86, 0.94, 1.0);

pub struct App {
    world: World,
    sprites: Sprites,
    camera: GameCamera,
    effects: Effects,
    input: InputState,
    previous: WorldSnapshot,
    current: WorldSnapshot,
    last_command: PlayerCommand,
    accumulator: f32,
    hint_timer: f32,
    save_toast: Option<(String, f32)>,
    time: f32,
}

impl App {
    pub fn new() -> Result<Self, LevelError> {
        let mut world = World::new(LEVEL_1)?;

        if let Some(data) = save::load() {
            world.load_save(&data);
        }

        let current = world.snapshot();
        let mut camera = GameCamera::new();
        camera.resize(vec2(screen_width(), screen_height()));
        camera.update(0.0, &current);

        Ok(Self {
            world,
            sprites: Sprites::load(),
            camera,
            effects: Effects::new(),
            input: InputState::default(),
            previous: current.clone(),
            current,
            last_command: PlayerCommand::none(),
            accumulator: 0.0,
            hint_timer: HINT_TIME,
            save_toast: None,
            time: 0.0,
        })
    }

    pub async fn run(&mut self) {
        loop {
            let frame_dt = get_frame_time().clamp(0.0, MAX_FRAME_TIME);
            self.time += frame_dt;

            self.camera.resize(vec2(screen_width(), screen_height()));
            self.input.poll();
            if self.input.has_activity() {
                self.hint_timer = 0.0;
            }

            self.advance(frame_dt);

            self.effects.update(frame_dt);
            self.camera.update(frame_dt, &self.current);

            self.hint_timer = (self.hint_timer - frame_dt).max(0.0);
            self.tick_toast(frame_dt);

            let alpha = (self.accumulator / DT).clamp(0.0, 1.0);
            render::draw_background(&self.camera, self.time);
            render::draw_world(&render::RenderContext {
                sprites: &self.sprites,
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
            hud::draw(
                &self.current,
                &self.effects,
                self.time,
                self.hint_timer,
                toast,
            );

            next_frame().await;
        }
    }

    /// Consome o tempo acumulado em ticks fixos. O primeiro tick do frame pode
    /// usar edges; os extras só repetem estados contínuos.
    fn advance(&mut self, frame_dt: f32) {
        self.accumulator = (self.accumulator + frame_dt).min(MAX_FRAME_TIME);

        let mut first_tick = true;
        while self.accumulator >= DT {
            let command = if first_tick {
                self.input.take_command()
            } else {
                self.last_command.without_edges()
            };
            self.last_command = command;
            first_tick = false;

            self.previous = self.current.clone();
            let output = self.world.step(TickInput::single(command));
            self.current = self.world.snapshot();
            self.handle_events(&output.events);

            self.accumulator -= DT;
        }
    }

    fn handle_events(&mut self, events: &[GameEvent]) {
        for event in events {
            match *event {
                GameEvent::MeleeHit { target, damage, .. } => {
                    if let Some(pos) = entity_center(&self.current, target) {
                        self.effects.damage(pos, damage, AURA);
                        self.effects.sparks(pos, AURA, 6);
                    }
                    self.camera.add_shake(2.0);
                }
                GameEvent::MogarFired { owner, .. } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        self.effects.sparks(pos, PURPLE, 10);
                        self.effects.text(pos, "MOGAR!", PURPLE, 12.0);
                    }
                    self.camera.add_shake(1.5);
                }
                GameEvent::MogarNoAura { owner } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        self.effects.no_aura(pos);
                    }
                    self.camera.add_shake(1.0);
                }
                GameEvent::MogarHit { target, damage, .. } => {
                    if let Some(pos) = entity_center(&self.current, target) {
                        self.effects.damage(pos, damage, PURPLE);
                        self.effects.sparks(pos, PURPLE, 12);
                    }
                    self.camera.add_shake(4.0);
                }
                GameEvent::MogarBlocked { owner } => {
                    if let Some(pos) = entity_center(&self.current, owner) {
                        self.effects.text(pos, "BLOQUEADO", STEEL, 10.0);
                        self.effects.sparks(pos, STEEL, 5);
                    }
                    self.camera.add_shake(1.5);
                }
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
                        self.effects
                            .text(pos, format!("NÍVEL {level}!"), GOLD, 16.0);
                        self.effects.sparks(pos, GOLD, 18);
                    }
                    self.effects.aura_flash = 0.7;
                    self.camera.add_shake(2.0);
                }
                GameEvent::PlayerDied { player } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(pos, "VOCÊ CAIU!", RED, 18.0);
                        self.effects.sparks(pos, RED, 16);
                    }
                    self.effects.hurt_flash = 0.7;
                    self.camera.add_shake(9.0);
                    self.persist("morte");
                }
                GameEvent::Victory { player } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(pos, "FASE CONCLUÍDA!", GOLD, 20.0);
                        self.effects.sparks(pos, GOLD, 24);
                    }
                    self.effects.victory_flash = 1.2;
                    self.camera.add_shake(4.0);
                }
                GameEvent::AttributeAllocated { player, attribute } => {
                    if let Some(pos) = entity_center(&self.current, player) {
                        self.effects.text(
                            pos,
                            format!("+{}", hud::attribute_name(attribute)),
                            AURA,
                            11.0,
                        );
                    }
                    self.persist("atributo");
                }
                GameEvent::PhaseRestarted { .. } => {
                    self.camera.reset();
                    self.effects.clear();
                }
                GameEvent::SaveRequested => self.persist("progresso"),
            }
        }
    }

    fn persist(&mut self, reason: &str) {
        let Some(data) = self.world.save_data() else {
            return;
        };
        match save::store(&data) {
            Ok(()) => self.save_toast = Some((reason.to_owned(), SAVE_TOAST_TIME)),
            Err(err) => {
                warn!("save: {err}");
                self.save_toast = Some(("falha ao salvar".to_owned(), SAVE_TOAST_TIME));
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

/// Centro visual de uma entidade no snapshot atual.
fn entity_center(snapshot: &WorldSnapshot, id: EntityId) -> Option<Vec2> {
    snapshot
        .entities
        .iter()
        .find(|entity| entity.id == id)
        .map(|entity| to_vec2(entity.pos + entity.size * 0.5))
}
