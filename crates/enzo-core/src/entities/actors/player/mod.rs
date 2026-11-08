//! Playable entity data.
//!
//! A playable entity has [`PlayerData`] and a
//! [`crate::entities::Controller::Player`] carrying per-tick input. This module
//! stores progression, resources, character identity, and customization. The
//! controller intent lives in `controller.rs`.
//!
//! Character attributes are not stored here: they are an [`crate::entities::Entity`]
//! component so equipment can contribute to effective values.
//!
//! Concrete playable character specs live in actor modules such as `enzo.rs` and
//! `valentina.rs`.

use crate::math::Vec2;

use crate::entities::base::attributes::Stats;
use crate::entities::item::ItemKind;

pub type PlayerId = u32;

/// Selects which playable character a player controls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlayableCharacter {
    #[default]
    Enzo,
    Valentina,
}

impl PlayableCharacter {
    pub const ALL: [Self; 2] = [Self::Enzo, Self::Valentina];

    pub const fn code(self) -> u8 {
        self as u8
    }

    pub fn from_code(code: u8) -> Option<Self> {
        Self::ALL.into_iter().find(|c| c.code() == code)
    }

    pub const fn id(self) -> &'static str {
        match self {
            Self::Enzo => "enzo",
            Self::Valentina => "valentina",
        }
    }

    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Enzo => "Enzo",
            Self::Valentina => "Valentina",
        }
    }
}

/// Stable visual customization IDs (no image paths in core).
/// Values are interpreted by the client/asset manifest.
/// Layers: skin/body, face, hair (style+color), outfit top/shirt, outfit bottom/pants, shoes.
/// Equipment items may override or add layers at render time.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VisualProfile {
    #[cfg_attr(feature = "serde", serde(default))]
    pub skin_tone: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub hair_style: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub hair_color: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub face: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub outfit_top: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub outfit_bottom: u8,
    #[cfg_attr(feature = "serde", serde(default))]
    pub shoes: u8,
}

impl VisualProfile {
    pub const FIELD_SKIN_TONE: u8 = 0;
    pub const FIELD_HAIR_STYLE: u8 = 1;
    pub const FIELD_HAIR_COLOR: u8 = 2;
    pub const FIELD_FACE: u8 = 3;
    pub const FIELD_OUTFIT_TOP: u8 = 4;
    pub const FIELD_OUTFIT_BOTTOM: u8 = 5;
    pub const FIELD_SHOES: u8 = 6;

    /// Sensible defaults per character (hair/face indices map to gender-exclusive catalogs).
    pub fn default_for(character: PlayableCharacter) -> Self {
        match character {
            // Enzo: short masculine hair by default (index 0 is bald option).
            PlayableCharacter::Enzo => Self {
                skin_tone: 0,
                hair_style: 1,
                hair_color: 0,
                face: 0,
                outfit_top: 0,
                outfit_bottom: 0,
                shoes: 0,
            },
            // Valentina: feminine default.
            PlayableCharacter::Valentina => Self {
                skin_tone: 0,
                hair_style: 0,
                hair_color: 1,
                face: 0,
                outfit_top: 1,
                outfit_bottom: 1,
                shoes: 0,
            },
        }
    }

    pub fn with_field(mut self, field: u8, value: u8) -> Self {
        match field {
            Self::FIELD_SKIN_TONE => self.skin_tone = value,
            Self::FIELD_HAIR_STYLE => self.hair_style = value.min(7),
            Self::FIELD_HAIR_COLOR => self.hair_color = value,
            Self::FIELD_FACE => self.face = value.min(7),
            Self::FIELD_OUTFIT_TOP => self.outfit_top = value,
            Self::FIELD_OUTFIT_BOTTOM => self.outfit_bottom = value,
            Self::FIELD_SHOES => self.shoes = value,
            _ => {}
        }
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerData {
    pub player_id: PlayerId,
    #[cfg_attr(feature = "serde", serde(default))]
    pub character: PlayableCharacter,
    #[cfg_attr(feature = "serde", serde(default))]
    pub visual_profile: VisualProfile,
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub aura: f32,
    pub pending_level_up: bool,
    pub respawn: Vec2,
    /// Collected items not yet equipped. The HUD reads this list from snapshots;
    /// the client must not mutate it directly.
    pub inventory: Vec<ItemKind>,
}

impl PlayerData {
    pub fn new(
        player_id: PlayerId,
        character: PlayableCharacter,
        visual_profile: VisualProfile,
        respawn: Vec2,
        stats: Stats,
    ) -> Self {
        Self {
            player_id,
            character,
            visual_profile,
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
