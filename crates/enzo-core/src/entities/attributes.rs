//! Atributos do personagem, atributos derivados e o conjunto genérico usado por
//! requisitos de movimentos.
//!
//! O personagem distribui pontos em [`Attributes`]. Equipamentos entram como
//! modificadores em [`AttributeSet`], então um movimento pode exigir, por
//! exemplo, `Força >= 2` do personagem **e** `Melee >= 1` vindo de uma espada.

use crate::input::Attribute;

/// Identidade extensível de um atributo. Novos atributos entram como novas
/// constantes; o conjunto é um array determinístico, sem `HashMap`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeId(pub u8);

impl AttributeId {
    pub const SPEED: Self = Self(0);
    pub const STRENGTH: Self = Self(1);
    pub const HEALTH: Self = Self(2);
    pub const AURA: Self = Self(3);
    pub const MELEE: Self = Self(4);
    pub const RANGED: Self = Self(5);

    pub const COUNT: usize = 6;
    pub const ALL: [Self; Self::COUNT] = [
        Self::SPEED,
        Self::STRENGTH,
        Self::HEALTH,
        Self::AURA,
        Self::MELEE,
        Self::RANGED,
    ];

    pub fn index(self) -> Option<usize> {
        let index = self.0 as usize;
        (index < Self::COUNT).then_some(index)
    }
}

/// Soma de atributos de todas as fontes (personagem + equipamentos + buffs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeSet {
    values: [i32; AttributeId::COUNT],
}

impl AttributeSet {
    pub const ZERO: Self = Self {
        values: [0; AttributeId::COUNT],
    };

    pub const fn new(values: [i32; AttributeId::COUNT]) -> Self {
        Self { values }
    }

    pub fn get(self, id: AttributeId) -> i32 {
        id.index().map(|index| self.values[index]).unwrap_or(0)
    }

    pub fn add(&mut self, id: AttributeId, amount: i32) {
        if let Some(index) = id.index() {
            self.values[index] += amount;
        }
    }

    /// Versão `const` de [`Self::add`], usada nos `REQUIREMENTS` dos movimentos.
    pub const fn with(mut self, id: AttributeId, amount: i32) -> Self {
        let index = id.0 as usize;
        if index < AttributeId::COUNT {
            self.values[index] += amount;
        }
        self
    }

    /// `true` se cada atributo deste conjunto alcança o mínimo exigido.
    pub fn meets_requirements(self, required: Self) -> bool {
        (0..AttributeId::COUNT).all(|index| self.values[index] >= required.values[index])
    }

    pub fn merged(self, other: Self) -> Self {
        let mut merged = self;
        for index in 0..AttributeId::COUNT {
            merged.values[index] += other.values[index];
        }
        merged
    }

    /// `true` se todos os pares `(atributo, mínimo)` forem alcançados.
    pub fn meets(self, required: &[(AttributeId, i32)]) -> bool {
        required
            .iter()
            .all(|(id, minimum)| self.get(*id) >= *minimum)
    }
}

/// Pontos investidos em cada atributo de personagem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attributes {
    pub speed: u32,
    pub strength: u32,
    pub health: u32,
    pub aura: u32,
}

impl Attributes {
    pub const ZERO: Self = Self {
        speed: 0,
        strength: 0,
        health: 0,
        aura: 0,
    };

    pub fn get(self, attribute: Attribute) -> u32 {
        match attribute {
            Attribute::Speed => self.speed,
            Attribute::Strength => self.strength,
            Attribute::Health => self.health,
            Attribute::Aura => self.aura,
        }
    }

    pub fn add(&mut self, attribute: Attribute) {
        match attribute {
            Attribute::Speed => self.speed += 1,
            Attribute::Strength => self.strength += 1,
            Attribute::Health => self.health += 1,
            Attribute::Aura => self.aura += 1,
        }
    }

    /// Converte os pontos do personagem para o conjunto genérico usado nos
    /// requisitos. Equipamentos são somados depois.
    pub fn to_set(self) -> AttributeSet {
        AttributeSet::ZERO
            .with(AttributeId::SPEED, self.speed as i32)
            .with(AttributeId::STRENGTH, self.strength as i32)
            .with(AttributeId::HEALTH, self.health as i32)
            .with(AttributeId::AURA, self.aura as i32)
    }
}

/// Atributos derivados, iguais para qualquer entidade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub strength: f32,
    pub max_health: f32,
    pub max_aura: f32,
    pub mogar_damage: f32,
}

impl Stats {
    pub fn player(attributes: Attributes) -> Self {
        let speed = attributes.speed as f32;
        Self {
            walk_speed: 220.0 + 10.0 * speed,
            run_speed: 340.0 + 6.0 * speed,
            jump_speed: 560.0 + 2.0 * speed,
            strength: 2.0 * attributes.strength as f32,
            max_health: 100.0 + 10.0 * attributes.health as f32,
            max_aura: 100.0 + 10.0 * attributes.aura as f32,
            mogar_damage: 20.0 + 2.0 * attributes.aura as f32,
        }
    }

    /// `hit_index` é 0, 1 ou 2 (primeiro, segundo e terceiro golpe).
    pub fn melee_damage(&self, hit_index: usize) -> f32 {
        let base = if hit_index >= 2 { 12.0 } else { 6.0 };
        base + self.strength
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn character_attributes_convert_to_set() {
        let attributes = Attributes {
            speed: 1,
            strength: 2,
            health: 3,
            aura: 4,
        };
        let set = attributes.to_set();
        assert_eq!(set.get(AttributeId::SPEED), 1);
        assert_eq!(set.get(AttributeId::STRENGTH), 2);
        assert_eq!(set.get(AttributeId::HEALTH), 3);
        assert_eq!(set.get(AttributeId::AURA), 4);
        assert_eq!(set.get(AttributeId::MELEE), 0);
    }

    #[test]
    fn attribute_sets_merge_and_meet_requirements() {
        let character = AttributeSet::ZERO
            .with(AttributeId::STRENGTH, 2)
            .with(AttributeId::AURA, 1);
        let equipment = AttributeSet::ZERO.with(AttributeId::MELEE, 1);
        let totals = character.merged(equipment);

        assert!(totals.meets(&[(AttributeId::STRENGTH, 2), (AttributeId::MELEE, 1)]));
        assert!(!totals.meets(&[(AttributeId::MELEE, 2)]));
        assert!(totals.meets(&[]));
    }

    #[test]
    fn attribute_set_meets_a_minimum_set() {
        let totals = AttributeSet::ZERO
            .with(AttributeId::STRENGTH, 2)
            .with(AttributeId::MELEE, 1);
        assert!(totals.meets_requirements(
            AttributeSet::ZERO
                .with(AttributeId::STRENGTH, 2)
                .with(AttributeId::MELEE, 1)
        ));
        assert!(!totals.meets_requirements(AttributeSet::ZERO.with(AttributeId::AURA, 1)));
        assert!(totals.meets_requirements(AttributeSet::ZERO));
    }

    #[test]
    fn stats_scale_with_attributes() {
        let attributes = Attributes {
            speed: 2,
            strength: 3,
            health: 1,
            aura: 4,
        };
        let stats = Stats::player(attributes);
        assert_eq!(stats.walk_speed, 240.0);
        assert_eq!(stats.run_speed, 352.0);
        assert_eq!(stats.jump_speed, 564.0);
        assert_eq!(stats.strength, 6.0);
        assert_eq!(stats.max_health, 110.0);
        assert_eq!(stats.max_aura, 140.0);
        assert_eq!(stats.mogar_damage, 28.0);
        assert_eq!(stats.melee_damage(0), 12.0);
        assert_eq!(stats.melee_damage(2), 18.0);
    }
}
