//! Dados de entidades jogáveis.
//!
//! Uma entidade jogável tem [`PlayerData`] e um
//! [`crate::entities::Controller::Player`] com o input do tick. Aqui ficam só
//! progressão e recursos; a intenção do controlador vive em `controller.rs`.
//!
//! Os atributos do personagem não ficam aqui: eles são um componente do
//! [`crate::entities::Entity`] para poderem ser somados ao equipamento.
//!
//! A entidade concreta do MVP (Enzo) e sua construção ficam em `enzo.rs`.

use crate::math::Vec2;

use crate::entities::base::attributes::Stats;
use crate::entities::item::ItemKind;

pub type PlayerId = u32;

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerData {
    pub player_id: PlayerId,
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub aura: f32,
    pub pending_level_up: bool,
    pub respawn: Vec2,
    /// Itens coletados e ainda não equipados. O HUD lê esta lista pelo snapshot;
    /// o cliente não pode alterá-la diretamente.
    pub inventory: Vec<ItemKind>,
}

impl PlayerData {
    pub fn new(player_id: PlayerId, respawn: Vec2, stats: Stats) -> Self {
        Self {
            player_id,
            level: 1,
            xp: 0.0,
            points: 0,
            aura: stats.max_aura,
            pending_level_up: false,
            respawn,
            inventory: Vec::new(),
        }
    }

    pub fn add_item(&mut self, kind: ItemKind) {
        self.inventory.push(kind);
    }

    pub fn remove_item(&mut self, kind: ItemKind) -> bool {
        let Some(index) = self.inventory.iter().position(|item| *item == kind) else {
            return false;
        };
        self.inventory.remove(index);
        true
    }
}
