//! Componentes de movimento.
//!
//! Todo movimento inerente é um componente independente dentro de
//! [`Movement`]: andar/correr, pular, dash e voar. O controlador (teclado ou
//! IA) só define intenção; a regra de aceleração, pulo e colisão fica aqui e em
//! [`crate::physics`].
//!
//! Novos movimentos entram como um novo componente em `Movement` (com estado e
//! configuração próprios) e uma etapa em [`Movement::update`]. Não se adiciona
//! lógica de movimento direto no `World` nem no cliente.

use crate::entity::{Body, Stats};
use crate::input::PlayerCommand;
use crate::map::Level;
use crate::math::approach;
use crate::physics;

pub const COYOTE_TIME: f32 = 0.10;
pub const JUMP_BUFFER_TIME: f32 = 0.12;
pub const RUN_TAP_WINDOW: f32 = 0.25;
pub const DASH_SPEED: f32 = 520.0;
pub const DASH_DURATION: f32 = 0.14;
pub const DASH_COOLDOWN: f32 = 0.70;
pub const FLY_SPEED: f32 = 180.0;
pub const FLY_ACCEL: f32 = 900.0;

/// Andar e correr por double-tap.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Walk {
    pub enabled: bool,
    pub intent_x: i8,
    pub running: bool,
    pub last_move_x: i8,
    pub last_tap_dir: i8,
    pub tap_timer: f32,
}

impl Walk {
    fn disabled() -> Self {
        Self {
            enabled: false,
            intent_x: 0,
            running: false,
            last_move_x: 0,
            last_tap_dir: 0,
            tap_timer: 0.0,
        }
    }
}

/// Pulo com coyote time, buffer e altura variável.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Jump {
    pub enabled: bool,
    pub held: bool,
    pub coyote: f32,
    pub buffer: f32,
    pub cut_pending: bool,
}

impl Jump {
    fn disabled() -> Self {
        Self {
            enabled: false,
            held: false,
            coyote: 0.0,
            buffer: 0.0,
            cut_pending: false,
        }
    }
}

/// Impulso horizontal com duração e cooldown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dash {
    pub enabled: bool,
    pub pending: bool,
    pub direction: i8,
    pub timer: f32,
    pub cooldown: f32,
}

impl Dash {
    fn disabled() -> Self {
        Self {
            enabled: false,
            pending: false,
            direction: 1,
            timer: 0.0,
            cooldown: 0.0,
        }
    }
}

/// Voo livre. Desabilitado no MVP do jogador, mas pronto para inimigos/itens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fly {
    pub enabled: bool,
    pub vertical_intent: i8,
}

impl Fly {
    fn disabled() -> Self {
        Self {
            enabled: false,
            vertical_intent: 0,
        }
    }
}

/// Componente de movimento presente em toda entidade.
///
/// Jogador habilita andar, pular e dash; inimigo habilita só andar; projétil
/// não usa nenhum movimento (a velocidade vem do `EntitySpec`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Movement {
    pub walk: Walk,
    pub jump: Jump,
    pub dash: Dash,
    pub fly: Fly,
}

impl Default for Movement {
    fn default() -> Self {
        Self::none()
    }
}

impl Movement {
    pub fn none() -> Self {
        Self {
            walk: Walk::disabled(),
            jump: Jump::disabled(),
            dash: Dash::disabled(),
            fly: Fly::disabled(),
        }
    }

    pub fn player() -> Self {
        Self {
            walk: Walk {
                enabled: true,
                ..Walk::disabled()
            },
            jump: Jump {
                enabled: true,
                ..Jump::disabled()
            },
            dash: Dash {
                enabled: true,
                ..Dash::disabled()
            },
            fly: Fly::disabled(),
        }
    }

    pub fn enemy() -> Self {
        Self {
            walk: Walk {
                enabled: true,
                ..Walk::disabled()
            },
            ..Self::none()
        }
    }

    /// Habilita voo. Usado por testes e por inimigos voadores futuros.
    pub fn with_fly(mut self) -> Self {
        self.fly.enabled = true;
        self
    }

    pub fn is_dashing(&self) -> bool {
        self.dash.enabled && self.dash.timer > 0.0
    }

    /// Traduz o comando do jogador em intenção. O edge de pulo/dash vira
    /// buffer/pedido; a execução acontece em [`Self::update`].
    pub fn apply_command(&mut self, command: PlayerCommand) {
        self.walk.intent_x = command.move_x.clamp(-1, 1);
        self.jump.held = command.jump_held;
        if command.jump && self.jump.enabled {
            self.jump.buffer = JUMP_BUFFER_TIME;
        }
        if command.dash && self.dash.enabled {
            self.dash.pending = true;
        }
    }

    /// Avança os componentes de movimento e integra o corpo no mapa.
    pub fn update(
        &mut self,
        body: &mut Body,
        stats: &Stats,
        level: &Level,
        speed_factor: f32,
        dt: f32,
    ) {
        self.tick_timers(body, dt);
        self.start_dash(body);
        self.update_jump(body, stats);
        self.update_walk(body, stats, speed_factor, dt);
        self.update_vertical(body, dt);
        physics::integrate(body, level, dt);

        if body.on_ground {
            self.jump.coyote = COYOTE_TIME;
        }
    }

    fn tick_timers(&mut self, body: &Body, dt: f32) {
        if body.on_ground {
            self.jump.coyote = COYOTE_TIME;
        } else {
            self.jump.coyote = (self.jump.coyote - dt).max(0.0);
        }
        self.jump.buffer = (self.jump.buffer - dt).max(0.0);

        self.walk.tap_timer = (self.walk.tap_timer - dt).max(0.0);
        if self.walk.tap_timer <= 0.0 {
            self.walk.last_tap_dir = 0;
        }

        self.dash.cooldown = (self.dash.cooldown - dt).max(0.0);
        self.dash.timer = (self.dash.timer - dt).max(0.0);
    }

    fn start_dash(&mut self, body: &mut Body) {
        if !self.dash.enabled || !self.dash.pending {
            return;
        }
        self.dash.pending = false;
        if self.dash.cooldown > 0.0 || self.dash.timer > 0.0 {
            return;
        }

        let direction = if self.walk.intent_x != 0 {
            self.walk.intent_x
        } else {
            body.facing
        };
        self.dash.direction = direction;
        self.dash.timer = DASH_DURATION;
        self.dash.cooldown = DASH_COOLDOWN;
        body.facing = direction;
    }

    fn update_jump(&mut self, body: &mut Body, stats: &Stats) {
        if !self.jump.enabled {
            return;
        }

        if self.jump.buffer > 0.0 && self.jump.coyote > 0.0 {
            body.vel.y = -stats.jump_speed;
            body.on_ground = false;
            self.jump.buffer = 0.0;
            self.jump.coyote = 0.0;
            self.jump.cut_pending = true;
        }

        if body.vel.y >= 0.0 {
            self.jump.cut_pending = false;
        } else if self.jump.cut_pending && !self.jump.held {
            body.vel.y *= 0.5;
            self.jump.cut_pending = false;
        }
    }

    fn update_walk(&mut self, body: &mut Body, stats: &Stats, speed_factor: f32, dt: f32) {
        if !self.walk.enabled {
            return;
        }

        self.update_run_state();

        if self.is_dashing() {
            body.vel.x = self.dash.direction as f32 * DASH_SPEED;
            return;
        }

        if self.walk.intent_x != 0 {
            body.facing = self.walk.intent_x;
        }

        let base = if self.walk.running {
            stats.run_speed
        } else {
            stats.walk_speed
        };
        let target = self.walk.intent_x as f32 * base * speed_factor;
        physics::horizontal_step(body, target, body.on_ground, dt);
    }

    /// Detecta duas pressões na mesma direção. Segurar mantém a corrida; soltar,
    /// parar ou inverter cancela.
    fn update_run_state(&mut self) {
        if self.walk.intent_x == 0 {
            self.walk.running = false;
            self.walk.last_move_x = 0;
            return;
        }

        let previous = self.walk.last_move_x;
        if previous != 0 && previous != self.walk.intent_x {
            self.walk.running = false;
        }

        if self.walk.intent_x != previous {
            if self.walk.last_tap_dir == self.walk.intent_x && self.walk.tap_timer > 0.0 {
                self.walk.running = true;
                self.walk.last_tap_dir = 0;
                self.walk.tap_timer = 0.0;
            } else {
                self.walk.last_tap_dir = self.walk.intent_x;
                self.walk.tap_timer = RUN_TAP_WINDOW;
            }
        }

        self.walk.last_move_x = self.walk.intent_x;
    }

    fn update_vertical(&mut self, body: &mut Body, dt: f32) {
        if self.is_dashing() {
            // O dash é horizontal; zera a queda no primeiro tick para não
            // "escorregar" da borda.
            body.vel.y = body.vel.y.min(0.0);
            return;
        }

        if self.fly.enabled {
            let target = self.fly.vertical_intent as f32 * FLY_SPEED;
            body.vel.y = approach(body.vel.y, target, FLY_ACCEL * dt);
            body.on_ground = false;
            return;
        }

        physics::apply_gravity(body, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntitySpec;
    use crate::input::PlayerCommand;
    use crate::math::Vec2;

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    fn command(move_x: i8, jump: bool, jump_held: bool, dash: bool) -> PlayerCommand {
        PlayerCommand {
            move_x,
            jump,
            jump_held,
            dash,
            ..PlayerCommand::none()
        }
    }

    fn stats() -> Stats {
        Stats::player(Default::default())
    }

    fn body_at(x: f32, y: f32) -> Body {
        Body::new(Vec2::new(x, y), crate::entity::PLAYER_SIZE)
    }

    #[test]
    fn double_tap_starts_running() {
        let mut movement = Movement::player();
        movement.apply_command(command(1, false, false, false));
        movement.update_run_state();
        assert!(!movement.walk.running);
        movement.apply_command(command(0, false, false, false));
        movement.update_run_state();
        movement.apply_command(command(1, false, false, false));
        movement.update_run_state();
        assert!(movement.walk.running);
    }

    #[test]
    fn releasing_or_reversing_cancels_run() {
        let mut movement = Movement::player();
        for intent in [1, 0, 1] {
            movement.walk.intent_x = intent;
            movement.update_run_state();
        }
        assert!(movement.walk.running);

        movement.walk.intent_x = 0;
        movement.update_run_state();
        assert!(!movement.walk.running);

        for intent in [1, 0, 1] {
            movement.walk.intent_x = intent;
            movement.update_run_state();
        }
        assert!(movement.walk.running);
        movement.walk.intent_x = -1;
        movement.update_run_state();
        assert!(!movement.walk.running);
    }

    #[test]
    fn walk_reaches_stats_speed() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player();
        let stats = stats();

        for _ in 0..120 {
            movement.apply_command(command(1, false, false, false));
            movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        }

        assert_eq!(body.vel.x, stats.walk_speed);
    }

    #[test]
    fn farming_speed_factor_slows_walk() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player();
        let stats = stats();

        for _ in 0..120 {
            movement.apply_command(command(1, false, false, false));
            movement.update(&mut body, &stats, &level, 0.45, crate::DT);
        }

        assert!((body.vel.x - stats.walk_speed * 0.45).abs() < 0.01);
    }

    #[test]
    fn jump_only_when_grounded_and_cuts_on_release() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 2.0);
        let mut movement = Movement::player();
        let stats = stats();

        // Primeiro tick pousa no chão.
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        assert!(body.on_ground);

        movement.apply_command(command(0, true, true, false));
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        assert!(body.vel.y < 0.0);
        let rising = body.vel.y;

        movement.apply_command(command(0, false, false, false));
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        assert!(body.vel.y > rising);
    }

    #[test]
    fn coyote_time_allows_jump_after_leaving_ledge() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 2.0);
        let mut movement = Movement::player();
        let stats = stats();
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);

        // Sai da borda: fica sem chão, mas ainda dentro do coyote.
        body.pos.x = 200.0;
        movement.apply_command(command(0, true, true, false));
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        assert!(body.vel.y < 0.0);
    }

    #[test]
    fn dash_bursts_and_respects_cooldown() {
        let level = level("P.........\n##########\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player();
        let stats = stats();
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);

        movement.apply_command(command(1, false, false, true));
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        assert!(movement.is_dashing());
        assert_eq!(body.vel.x, DASH_SPEED);

        for _ in 0..10 {
            movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        }
        assert!(!movement.is_dashing());
        assert!(movement.dash.cooldown > 0.0);

        movement.apply_command(command(1, false, false, true));
        movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        assert!(!movement.is_dashing());
    }

    #[test]
    fn fly_disables_gravity_and_moves_vertically() {
        let level = level("P...\n....\n....\n####\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player().with_fly();
        let stats = stats();
        movement.fly.vertical_intent = -1;

        let start_y = body.pos.y;
        for _ in 0..30 {
            movement.update(&mut body, &stats, &level, 1.0, crate::DT);
        }
        assert!(body.pos.y < start_y);
        assert!(body.vel.y < 0.0);
    }

    #[test]
    fn enemy_has_only_walk() {
        let movement = Movement::enemy();
        assert!(movement.walk.enabled);
        assert!(!movement.jump.enabled);
        assert!(!movement.dash.enabled);
        assert!(!movement.fly.enabled);
        assert_eq!(EntitySpec::enemy(Vec2::ZERO).movement, movement);
    }

    #[test]
    fn projectile_has_no_movement_components() {
        let spec = EntitySpec::mogar(crate::entity::EntityId(0), Vec2::ZERO, 1, 20.0);
        assert_eq!(spec.movement, Movement::none());
    }
}
