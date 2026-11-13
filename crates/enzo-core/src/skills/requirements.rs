//! Requisitos de skill e contexto da entidade.
//!
//! A validação é feita apenas no core. O cliente recebe o [`SkillLock`] no
//! snapshot e se limita a mostrar o motivo (nível, atributo ou equipamento).

use crate::entities::{AttributeId, AttributeSet, EquipmentTags};

use super::{SkillId, SkillLevels};

/// Tipo de entidade usado para validar requisitos.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntityType {
    Playable,
    Enemy,
    Projectile,
    Item,
}

/// Contexto do jogador no momento da checagem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkillContext {
    pub entity_type: EntityType,
    pub level: u32,
    pub attributes: AttributeSet,
    pub equipment: EquipmentTags,
    #[cfg_attr(feature = "serde", serde(default))]
    pub skill_levels: SkillLevels,
}

impl SkillContext {
    pub const fn new(
        entity_type: EntityType,
        level: u32,
        attributes: AttributeSet,
        equipment: EquipmentTags,
        skill_levels: SkillLevels,
    ) -> Self {
        Self {
            entity_type,
            level,
            attributes,
            equipment,
            skill_levels,
        }
    }

    pub fn effective_skill_level(&self, skill: SkillId, base_skill_level: u8) -> u8 {
        base_skill_level.saturating_add(self.skill_levels[skill.index()])
    }
}

/// Primeiro requisito não cumprido. Guarda apenas valores `Copy`/serializáveis:
/// a lista de tipos aceitos fica no [`Requirements`], não no motivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkillLock {
    EntityType,
    SkillLevel {
        required: u8,
        current: u8,
    },
    Level {
        required: u32,
        current: u32,
    },
    Attribute {
        attribute: AttributeId,
        required: u16,
        current: u16,
    },
    Equipment {
        required: EquipmentTags,
        current: EquipmentTags,
    },
}

/// Regras declaradas por uma skill.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirements {
    pub entity_types: &'static [EntityType],
    pub min_level: u32,
    pub min_skill_level: u8,
    pub min_attributes: AttributeSet,
    pub required_equipment: EquipmentTags,
}

impl Requirements {
    pub const fn new(
        entity_types: &'static [EntityType],
        min_level: u32,
        min_attributes: AttributeSet,
        required_equipment: EquipmentTags,
    ) -> Self {
        Self {
            entity_types,
            min_level,
            min_skill_level: 0,
            min_attributes,
            required_equipment,
        }
    }

    /// Devolve o primeiro motivo de recusa, em ordem fixa: tipo de entidade,
    /// nível da skill, nível do jogador, atributos (ordem de [`AttributeId::ALL`])
    /// e equipamento.
    pub fn check(self, context: SkillContext, effective_skill_level: u8) -> Result<(), SkillLock> {
        if !self.entity_types.contains(&context.entity_type) {
            return Err(SkillLock::EntityType);
        }

        if effective_skill_level < self.min_skill_level {
            return Err(SkillLock::SkillLevel {
                required: self.min_skill_level,
                current: effective_skill_level,
            });
        }

        if context.level < self.min_level {
            return Err(SkillLock::Level {
                required: self.min_level,
                current: context.level,
            });
        }

        for attribute in AttributeId::ALL {
            let required = self.min_attributes.get(attribute);
            if required <= 0 {
                continue;
            }
            let current = context.attributes.get(attribute);
            if current < required {
                return Err(SkillLock::Attribute {
                    attribute,
                    required: required.min(u16::MAX as i32) as u16,
                    current: current.clamp(0, u16::MAX as i32) as u16,
                });
            }
        }

        if !context.equipment.contains(self.required_equipment) {
            return Err(SkillLock::Equipment {
                required: self.required_equipment,
                current: context.equipment,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{AttributeId, Attributes};
    use crate::skills::NO_SKILL_LEVELS;

    fn player_context(level: u32, attributes: Attributes) -> SkillContext {
        SkillContext::new(
            EntityType::Playable,
            level,
            attributes.to_set(),
            EquipmentTags::NONE,
            NO_SKILL_LEVELS,
        )
    }

    #[test]
    fn checks_entity_type_first() {
        let requirements = Requirements::new(
            &[EntityType::Playable],
            5,
            AttributeSet::ZERO,
            EquipmentTags::NONE,
        );
        assert_eq!(
            requirements.check(
                SkillContext::new(
                    EntityType::Enemy,
                    1,
                    AttributeSet::ZERO,
                    EquipmentTags::NONE,
                    NO_SKILL_LEVELS,
                ),
                1
            ),
            Err(SkillLock::EntityType)
        );
    }

    #[test]
    fn checks_skill_level_then_player_level() {
        let mut requirements = Requirements::new(
            &[EntityType::Playable],
            5,
            AttributeSet::ZERO,
            EquipmentTags::NONE,
        );
        requirements.min_skill_level = 2;
        assert_eq!(
            requirements.check(player_context(5, Attributes::ZERO), 1),
            Err(SkillLock::SkillLevel {
                required: 2,
                current: 1,
            })
        );
        assert_eq!(
            requirements.check(player_context(1, Attributes::ZERO), 2),
            Err(SkillLock::Level {
                required: 5,
                current: 1,
            })
        );
        assert!(requirements
            .check(player_context(5, Attributes::ZERO), 2)
            .is_ok());
    }

    #[test]
    fn checks_level_then_attributes() {
        let requirements = Requirements::new(
            &[EntityType::Playable],
            5,
            AttributeSet::ZERO.with(AttributeId::AURA, 5),
            EquipmentTags::NONE,
        );
        assert_eq!(
            requirements.check(player_context(1, Attributes::ZERO), 1),
            Err(SkillLock::Level {
                required: 5,
                current: 1,
            })
        );
        assert_eq!(
            requirements.check(player_context(5, Attributes::ZERO), 1),
            Err(SkillLock::Attribute {
                attribute: AttributeId::AURA,
                required: 5,
                current: 0,
            })
        );
        let strong = Attributes {
            aura: 5,
            ..Attributes::ZERO
        };
        assert!(requirements.check(player_context(5, strong), 1).is_ok());
    }

    #[test]
    fn checks_equipment_last() {
        let requirements = Requirements::new(
            &[EntityType::Playable],
            1,
            AttributeSet::ZERO,
            EquipmentTags::new(1 << 6),
        );
        assert_eq!(
            requirements.check(player_context(1, Attributes::ZERO), 1),
            Err(SkillLock::Equipment {
                required: EquipmentTags::new(1 << 6),
                current: EquipmentTags::NONE,
            })
        );
    }
}
