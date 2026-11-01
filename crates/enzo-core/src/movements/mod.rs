//! Componentes de movimento.
//!
//! Cada movimento inerente vive em seu próprio módulo (`walk`, `jump`, `dash`,
//! `fly`, `sixseven`) e declara seus `REQUIREMENTS`. [`Movement`] reúne os
//! componentes e o [`MoveSet`] atribuído à entidade.
//!
//! O controlador (teclado ou IA) só define intenção. Aceleração, pulo, dash,
//! SixSeven e colisão são regras do core. Novos movimentos entram como um
//! módulo novo, uma variante de `MoveId` e uma etapa em [`Movement::update`].

pub mod dash;
pub mod fly;
pub mod jump;
pub mod sixseven;
pub mod walk;

pub use dash::Dash;
pub use fly::Fly;
pub use jump::Jump;
pub use sixseven::SixSeven;
pub use walk::Walk;

use crate::entities::{Body, Stats};
use crate::input::PlayerCommand;
use crate::map::Level;
use crate::moves::{AbilityContext, AssignError, MoveCategory, MoveId, MoveSet};
use crate::physics;

/// Componente de movimento presente em toda entidade.
///
/// Os componentes existem sempre; o [`MoveSet`] diz quais estão atribuídos.
/// Jogador nasce com andar, pular, dash e SixSeven; inimigo nasce só com andar;
/// projétil não tem movimentos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Movement {
    pub moves: MoveSet,
    pub walk: Walk,
    pub jump: Jump,
    pub dash: Dash,
    pub fly: Fly,
    pub sixseven: SixSeven,
}

impl Default for Movement {
    fn default() -> Self {
        Self::none()
    }
}

impl Movement {
    pub fn none() -> Self {
        Self {
            moves: MoveSet::new(),
            walk: Walk::default(),
            jump: Jump::default(),
            dash: Dash::default(),
            fly: Fly::default(),
            sixseven: SixSeven::default(),
        }
    }

    pub fn player() -> Self {
        Self::from_moves(&[MoveId::Walk, MoveId::Jump, MoveId::Dash, MoveId::SixSeven])
    }

    pub fn enemy() -> Self {
        Self::from_moves(&[MoveId::Walk])
    }

    pub(crate) fn from_moves(ids: &[MoveId]) -> Self {
        let mut movement = Self::none();
        movement.moves = MoveSet::from_slice(ids);
        movement
    }

    pub fn has(&self, id: MoveId) -> bool {
        self.moves.contains(id)
    }

    /// Atribui um movimento se ele for da categoria correta e a entidade cumprir
    /// os requisitos declarados pelo movimento.
    pub fn try_assign(&mut self, id: MoveId, context: AbilityContext) -> Result<(), AssignError> {
        if id.category() != MoveCategory::Movement {
            return Err(AssignError::WrongCategory);
        }
        self.moves.try_assign(id, context)
    }

    pub fn with_move(mut self, id: MoveId, context: AbilityContext) -> Result<Self, AssignError> {
        self.try_assign(id, context)?;
        Ok(self)
    }

    pub fn is_dashing(&self) -> bool {
        self.has(MoveId::Dash) && self.dash.is_active()
    }

    /// Traduz o comando do jogador em intenção. O edge de pulo/dash/SixSeven
    /// vira buffer/pedido; a execução acontece em [`Self::update`].
    pub fn apply_command(&mut self, command: PlayerCommand) {
        self.walk.apply(command);
        if self.has(MoveId::Jump) {
            self.jump.apply(command);
        }
        if self.has(MoveId::Dash) && command.dash {
            self.dash.request();
        }
        if self.has(MoveId::SixSeven) {
            if let Some(key) = command.sixseven {
                self.sixseven.request(key);
            }
        }
    }

    /// Avança os componentes atribuídos e integra o corpo no mapa.
    ///
    /// A lentidão do SixSeven é calculada aqui, a partir do próprio componente,
    /// para o movimento não depender de estado externo.
    pub fn update(&mut self, body: &mut Body, stats: &Stats, level: &Level, dt: f32) {
        self.walk.tick(dt);
        self.jump.tick(body, dt);
        self.dash.tick(dt);
        self.sixseven.tick(dt);

        if self.has(MoveId::Dash) {
            self.dash.try_start(body, self.walk.intent_x);
        }
        if self.has(MoveId::Jump) {
            self.jump.update(body, stats);
        }

        let speed_factor = self.sixseven.speed_factor();
        self.update_horizontal(body, stats, speed_factor, dt);
        self.update_vertical(body, dt);
        physics::integrate(body, level, dt);

        if self.has(MoveId::Jump) && body.on_ground {
            self.jump.coyote = jump::COYOTE_TIME;
        }
    }

    fn update_horizontal(&mut self, body: &mut Body, stats: &Stats, speed_factor: f32, dt: f32) {
        if self.is_dashing() {
            body.vel.x = self.dash.direction as f32 * dash::DASH_SPEED;
            return;
        }

        if self.has(MoveId::Walk) {
            self.walk.update(body, stats, speed_factor, dt);
        }
    }

    fn update_vertical(&mut self, body: &mut Body, dt: f32) {
        if self.is_dashing() {
            // O dash é horizontal; impede que a queda no primeiro tick faça o
            // corpo "escorregar" para fora da borda.
            body.vel.y = body.vel.y.min(0.0);
            return;
        }

        if self.has(MoveId::Fly) {
            self.fly.update(body, dt);
            return;
        }

        physics::apply_gravity(body, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Attributes, EntityId, EntitySpec, TeamId};
    use crate::math::Vec2;
    use crate::moves::EntityType;

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
        Stats::player(Attributes::ZERO)
    }

    fn body_at(x: f32, y: f32) -> Body {
        Body::new(Vec2::new(x, y), crate::entities::ENZO_SIZE)
    }

    fn fly_context() -> AbilityContext {
        AbilityContext::player(
            5,
            Attributes {
                aura: 5,
                ..Attributes::ZERO
            },
        )
    }

    #[test]
    fn player_has_default_moves() {
        let movement = Movement::player();
        assert!(movement.has(MoveId::Walk));
        assert!(movement.has(MoveId::Jump));
        assert!(movement.has(MoveId::Dash));
        assert!(movement.has(MoveId::SixSeven));
        assert!(!movement.has(MoveId::Fly));
        assert!(!movement.has(MoveId::Melee));
    }

    #[test]
    fn enemy_has_only_walk() {
        let movement = Movement::enemy();
        assert!(movement.has(MoveId::Walk));
        assert!(!movement.has(MoveId::Jump));
        assert!(!movement.has(MoveId::Dash));
        assert!(!movement.has(MoveId::Fly));
        assert!(!movement.has(MoveId::SixSeven));
        assert_eq!(EntitySpec::slime(Vec2::ZERO).movement, movement);
    }

    #[test]
    fn projectile_has_no_movement_components() {
        let spec = EntitySpec::mogar(EntityId(0), Vec2::ZERO, 1, 20.0, TeamId::PLAYERS);
        assert_eq!(spec.movement, Movement::none());
        assert!(spec.movement.moves.is_empty());
    }

    #[test]
    fn assigning_fly_checks_requirements() {
        let mut movement = Movement::player();
        assert_eq!(
            movement.try_assign(MoveId::Fly, AbilityContext::player(1, Attributes::ZERO)),
            Err(AssignError::LevelTooLow {
                required: 5,
                current: 1,
            })
        );
        assert!(movement.try_assign(MoveId::Fly, fly_context()).is_ok());
        assert!(movement.has(MoveId::Fly));
    }

    #[test]
    fn enemy_cannot_receive_player_only_moves() {
        let mut movement = Movement::enemy();
        assert!(matches!(
            movement.try_assign(MoveId::Jump, AbilityContext::enemy()),
            Err(AssignError::WrongEntityType {
                current: EntityType::Enemy,
                ..
            })
        ));
        assert!(!movement.has(MoveId::Jump));
    }

    #[test]
    fn movement_rejects_attack_moves() {
        let mut movement = Movement::player();
        assert_eq!(
            movement.try_assign(MoveId::Melee, AbilityContext::player(1, Attributes::ZERO)),
            Err(AssignError::WrongCategory)
        );
    }

    #[test]
    fn walk_reaches_stats_speed() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player();
        let stats = stats();

        for _ in 0..120 {
            movement.apply_command(command(1, false, false, false));
            movement.update(&mut body, &stats, &level, crate::DT);
        }

        assert_eq!(body.vel.x, stats.walk_speed);
    }

    #[test]
    fn sixseven_slows_walk() {
        use crate::input::SixSevenKey;

        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player();
        let stats = stats();
        movement.sixseven.press(SixSevenKey::Six);

        for _ in 0..30 {
            movement.apply_command(command(1, false, false, false));
            movement.update(&mut body, &stats, &level, crate::DT);
        }

        assert!((body.vel.x - stats.walk_speed * sixseven::SLOW_FACTOR).abs() < 0.01);
    }

    #[test]
    fn jump_only_when_grounded_and_cuts_on_release() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 2.0);
        let mut movement = Movement::player();
        let stats = stats();

        // Primeiro tick pousa no chão.
        movement.update(&mut body, &stats, &level, crate::DT);
        assert!(body.on_ground);

        movement.apply_command(command(0, true, true, false));
        movement.update(&mut body, &stats, &level, crate::DT);
        assert!(body.vel.y < 0.0);
        let rising = body.vel.y;

        movement.apply_command(command(0, false, false, false));
        movement.update(&mut body, &stats, &level, crate::DT);
        assert!(body.vel.y > rising);
    }

    #[test]
    fn coyote_time_allows_jump_after_leaving_ledge() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 2.0);
        let mut movement = Movement::player();
        let stats = stats();
        movement.update(&mut body, &stats, &level, crate::DT);

        // Sai da borda: fica sem chão, mas ainda dentro do coyote.
        body.pos.x = 200.0;
        movement.apply_command(command(0, true, true, false));
        movement.update(&mut body, &stats, &level, crate::DT);
        assert!(body.vel.y < 0.0);
    }

    #[test]
    fn dash_bursts_and_respects_cooldown() {
        let level = level("P.........\n##########\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player();
        let stats = stats();
        movement.update(&mut body, &stats, &level, crate::DT);

        movement.apply_command(command(1, false, false, true));
        movement.update(&mut body, &stats, &level, crate::DT);
        assert!(movement.is_dashing());
        assert_eq!(body.vel.x, dash::DASH_SPEED);

        for _ in 0..10 {
            movement.update(&mut body, &stats, &level, crate::DT);
        }
        assert!(!movement.is_dashing());
        assert!(movement.dash.cooldown > 0.0);

        movement.apply_command(command(1, false, false, true));
        movement.update(&mut body, &stats, &level, crate::DT);
        assert!(!movement.is_dashing());
    }

    #[test]
    fn fly_disables_gravity_and_moves_vertically() {
        let level = level("P...\n....\n....\n####\n");
        let mut body = body_at(0.0, 0.0);
        let mut movement = Movement::player()
            .with_move(MoveId::Fly, fly_context())
            .unwrap();
        let stats = stats();
        movement.fly.vertical_intent = -1;

        let start_y = body.pos.y;
        for _ in 0..30 {
            movement.update(&mut body, &stats, &level, crate::DT);
        }
        assert!(body.pos.y < start_y);
        assert!(body.vel.y < 0.0);
    }
}
