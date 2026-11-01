//! Dados de entidades jogáveis.
//!
//! Uma entidade jogável tem [`PlayerData`] e um [`crate::entities::Controller::Player`].
//! O input chega como [`crate::input::PlayerCommand`]; o componente guarda só o
//! estado que o controlador precisa.
//!
//! Os atributos do personagem não ficam aqui: eles são um componente do
//! [`crate::entities::Entity`] para poderem ser somados ao equipamento.
//!
//! A entidade concreta do MVP (Enzo) e sua construção ficam em `enzo.rs`.

use crate::math::Vec2;

use super::attributes::Stats;

pub type PlayerId = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerInput {
    pub move_x: i8,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub dash_pressed: bool,
    pub melee_pressed: bool,
    pub mogar_pressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerData {
    pub player_id: PlayerId,
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub aura: f32,
    pub input: PlayerInput,
    pub pending_level_up: bool,
    pub respawn: Vec2,
}

impl PlayerData {
    pub fn new(player_id: PlayerId, respawn: Vec2, stats: Stats) -> Self {
        Self {
            player_id,
            level: 1,
            xp: 0.0,
            points: 0,
            aura: stats.max_aura,
            input: PlayerInput::default(),
            pending_level_up: false,
            respawn,
        }
    }
}
