//! Shared humanoid archetype helpers (composition-based).
//!
//! Enzo, Valentina, and future humanoid enemies/characters share common entity
//! assembly patterns. This module provides reusable builders; concrete actor
//! modules keep identity, visual metadata, and gameplay-specific overrides.

use crate::math::Vec2;
use crate::mechanics::Locomotion;
use crate::skills::Skills;

use crate::entities::actors::player::{PlayableCharacter, PlayerData, PlayerId, VisualProfile};
use crate::entities::base::attributes::{Attributes, Stats};
use crate::entities::base::components::{Health, TeamId};
use crate::entities::base::controller::Controller;
use crate::entities::base::entity::EntityKind;
use crate::entities::base::equipment::{EquipSlot, Equipment};
use crate::entities::spec::EntitySpec;

/// Configuration for a playable humanoid character spec.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayableHumanoidConfig {
    pub character: PlayableCharacter,
    pub visual_profile: VisualProfile,
    pub size: Vec2,
    pub attributes: Attributes,
    pub locomotion: Locomotion,
    pub skills: Skills,
}

impl PlayableHumanoidConfig {
    pub fn new(character: PlayableCharacter, size: Vec2) -> Self {
        Self {
            character,
            visual_profile: VisualProfile::default(),
            size,
            attributes: Attributes::default(),
            locomotion: Locomotion::player(),
            skills: Skills::player_default(),
        }
    }

    pub fn with_visual_profile(mut self, visual_profile: VisualProfile) -> Self {
        self.visual_profile = visual_profile;
        self
    }

    pub fn with_attributes(mut self, attributes: Attributes) -> Self {
        self.attributes = attributes;
        self
    }
}

impl EntitySpec {
    /// Builds a playable humanoid entity using composition (shared base for Enzo/Valentina).
    pub fn playable_humanoid(
        player_id: PlayerId,
        pos: Vec2,
        config: PlayableHumanoidConfig,
    ) -> Self {
        let stats = Stats::player(config.attributes);
        Self {
            kind: EntityKind::Player(PlayerData::new(
                player_id,
                config.character,
                config.visual_profile,
                pos,
                stats,
            )),
            controller: Controller::player(),
            team: TeamId::PLAYERS,
            pos,
            size: config.size,
            health: Health::full(stats.max_health),
            stats,
            attributes: config.attributes,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            locomotion: config.locomotion,
            skills: config.skills,
            velocity: Vec2::ZERO,
        }
    }
}
