//! Snapshot serializável do mundo.

use crate::entities::{
    AnimationState, Attributes, EnemySpecies, Entity, EntityId, EntityKind, EquipSlot, Equipment,
    ItemCategory, ItemKind, ItemUsage, PlayableCharacter, PlayerId, VisualProfile,
};
use crate::input::{ActionSlot, SixSevenKey};
use crate::math::{Rect, Vec2};
use crate::progression;
use crate::skills::{Loadout, SkillId, SkillLock};
use crate::state::GamePhase;

pub const SNAPSHOT_VERSION: u16 = 4;

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorldSnapshot {
    pub version: u16,
    pub tick: u64,
    pub phase: GamePhase,
    pub phase_timer: f32,
    pub entities: Vec<EntitySnapshot>,
    pub level_width: f32,
    pub level_height: f32,
    pub flag: Option<Rect>,
    pub camera_hint: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub kind: SnapshotKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
    pub facing: i8,
    pub on_ground: bool,
    pub dashing: bool,
    pub running: bool,
    pub team: u8,
    pub health: f32,
    pub max_health: f32,
    pub iframes: f32,
    #[cfg_attr(feature = "serde", serde(default))]
    pub animation: AnimationState,
    pub player: Option<PlayerSnapshot>,
    pub enemy: Option<EnemySnapshot>,
    pub projectile: Option<ProjectileSnapshot>,
    pub item: Option<ItemSnapshot>,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkillSlotSnapshot {
    pub skill: Option<SkillId>,
    pub available: bool,
    pub lock: Option<SkillLock>,
    pub cooldown: f32,
    pub cooldown_max: f32,
    pub aura_cost: f32,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SkillCatalogEntry {
    pub skill: SkillId,
    pub available: bool,
    pub lock: Option<SkillLock>,
    pub equipped_slot: Option<ActionSlot>,
    pub aura_cost: f32,
    pub cooldown_max: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerSnapshot {
    pub player_id: PlayerId,
    #[cfg_attr(feature = "serde", serde(default))]
    pub character: PlayableCharacter,
    #[cfg_attr(feature = "serde", serde(default))]
    pub visual_profile: VisualProfile,
    pub level: u32,
    pub xp: f32,
    pub xp_to_next: f32,
    pub points: u32,
    pub aura: f32,
    pub max_aura: f32,
    pub attributes: Attributes,
    pub equipment: Equipment,
    #[cfg_attr(
        feature = "serde",
        serde(default, deserialize_with = "de_equipped_slots")
    )]
    pub equipped: [Option<EquippedItemSnapshot>; EquipSlot::COUNT],
    pub inventory: Vec<ItemKind>,
    pub pending_level_up: bool,
    pub farming: bool,
    pub sixseven_last: Option<SixSevenKey>,
    pub sixseven_combo: u32,
    pub sixseven_window: f32,
    pub loadout: Loadout,
    pub slots: [SkillSlotSnapshot; ActionSlot::COUNT],
    pub catalog: Vec<SkillCatalogEntry>,
    pub combo_window: f32,
    pub melee_step: u8,
    pub melee_timer: f32,
}

#[cfg(feature = "serde")]
fn de_equipped_slots<'de, D>(
    deserializer: D,
) -> Result<[Option<EquippedItemSnapshot>; EquipSlot::COUNT], D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let items = Vec::<Option<EquippedItemSnapshot>>::deserialize(deserializer)?;
    let mut out = [None; EquipSlot::COUNT];
    for (i, item) in items.into_iter().take(EquipSlot::COUNT).enumerate() {
        out[i] = item;
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EquippedItemSnapshot {
    pub slot: EquipSlot,
    pub kind: ItemKind,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnemySnapshot {
    pub species: EnemySpecies,
    pub patrol_dir: i8,
    pub stun: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectileSnapshot {
    pub owner: EntityId,
    pub skill: SkillId,
    pub damage: f32,
    pub lifetime: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ItemSnapshot {
    pub kind: ItemKind,
    pub category: ItemCategory,
    pub usage: ItemUsage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SnapshotKind {
    Player,
    Enemy,
    Projectile,
    Item,
}

impl EntitySnapshot {
    pub fn from_entity(entity: &Entity) -> Self {
        Self::from_entity_at_tick(entity, 0)
    }

    pub fn from_entity_at_tick(entity: &Entity, tick: u64) -> Self {
        let kind = match entity.kind {
            EntityKind::Player(_) => SnapshotKind::Player,
            EntityKind::Enemy(_) => SnapshotKind::Enemy,
            EntityKind::Projectile(_) => SnapshotKind::Projectile,
            EntityKind::Item(_) => SnapshotKind::Item,
        };
        let alive = entity.health.is_alive();
        let farming = entity.locomotion.sixseven.is_active();
        let sixseven_last = entity.locomotion.sixseven.last_key;
        let melee_active = entity.skills.is_active(SkillId::Melee);
        let melee_step = entity.skills.melee.step;
        let flying = entity.skills.is_active(SkillId::Fly);
        let dashing = entity.skills.is_active(SkillId::Dash);
        let anim_time = tick as f32 / 60.0;
        let animation = AnimationState::resolve(
            alive,
            entity.body.facing,
            entity.body.on_ground,
            entity.body.vel.x,
            entity.body.vel.y,
            entity.locomotion.walk.running,
            dashing,
            flying,
            false,
            entity.health.iframes > 0.0 && alive,
            melee_step,
            melee_active,
            false,
            false,
            farming,
            sixseven_last,
            anim_time,
        );

        Self {
            id: entity.id,
            kind,
            pos: entity.body.pos,
            vel: entity.body.vel,
            size: entity.body.size,
            facing: entity.body.facing,
            on_ground: entity.body.on_ground,
            dashing,
            running: entity.locomotion.walk.running,
            team: entity.team.0,
            health: entity.health.current,
            max_health: entity.stats.max_health,
            iframes: entity.health.iframes,
            animation,
            player: entity
                .player()
                .map(|player| PlayerSnapshot::from_player(player, entity)),
            enemy: entity.enemy().map(|enemy| EnemySnapshot {
                species: enemy.species,
                patrol_dir: entity
                    .ai()
                    .map(|ai| ai.patrol_dir())
                    .unwrap_or(if entity.body.facing >= 0 { 1 } else { -1 }),
                stun: enemy.stun,
            }),
            projectile: entity.projectile().map(|projectile| ProjectileSnapshot {
                owner: projectile.owner,
                skill: projectile.skill,
                damage: projectile.damage,
                lifetime: projectile.lifetime,
            }),
            item: entity.item().map(|item| ItemSnapshot {
                kind: item.kind,
                category: item.category,
                usage: item.usage(),
            }),
        }
    }
}

impl PlayerSnapshot {
    pub fn from_player(player: &crate::entities::PlayerData, entity: &Entity) -> Self {
        let context = entity.skill_context();
        let mut slots = std::array::from_fn(|_| SkillSlotSnapshot {
            skill: None,
            available: false,
            lock: None,
            cooldown: 0.0,
            cooldown_max: 0.0,
            aura_cost: 0.0,
            active: false,
        });
        for (index, slot) in ActionSlot::ALL.iter().copied().enumerate() {
            let skill = entity.skills.loadout.get(slot);
            let (available, lock, cooldown_max, aura_cost, active, cooldown) =
                if let Some(skill) = skill {
                    let def = skill.def();
                    let lock = def.requirements.check(context).err();
                    (
                        lock.is_none(),
                        lock,
                        def.cooldown,
                        def.aura_cost,
                        entity.skills.is_active(skill),
                        entity.skills.cooldown(skill),
                    )
                } else {
                    (false, None, 0.0, 0.0, false, 0.0)
                };
            slots[index] = SkillSlotSnapshot {
                skill,
                available,
                lock,
                cooldown,
                cooldown_max,
                aura_cost,
                active,
            };
        }
        let catalog = SkillId::ALL
            .iter()
            .copied()
            .map(|skill| {
                let def = skill.def();
                let lock = def.requirements.check(context).err();
                SkillCatalogEntry {
                    skill,
                    available: lock.is_none(),
                    lock,
                    equipped_slot: entity.skills.loadout.slot_of(skill),
                    aura_cost: def.aura_cost,
                    cooldown_max: def.cooldown,
                }
            })
            .collect();
        Self {
            player_id: player.player_id,
            character: player.character,
            visual_profile: player.visual_profile,
            level: player.level,
            xp: player.xp,
            xp_to_next: progression::xp_to_next(player.level),
            points: player.points,
            aura: player.aura,
            max_aura: entity.stats.max_aura,
            attributes: entity.attributes,
            equipment: entity.equipment,
            equipped: std::array::from_fn(|index| {
                entity.equipped[index].map(|item| EquippedItemSnapshot {
                    slot: item.slot,
                    kind: item.kind,
                })
            }),
            inventory: player.inventory.clone(),
            pending_level_up: player.pending_level_up,
            farming: entity.locomotion.sixseven.is_active(),
            sixseven_last: entity.locomotion.sixseven.last_key,
            sixseven_combo: entity.locomotion.sixseven.combo,
            sixseven_window: entity.locomotion.sixseven.window,
            loadout: entity.skills.loadout,
            slots,
            catalog,
            combo_window: entity.skills.melee.window,
            melee_step: entity.skills.melee.step,
            melee_timer: entity.skills.melee.anim_timer,
        }
    }
}

impl WorldSnapshot {
    pub fn player(&self) -> Option<&EntitySnapshot> {
        self.entities
            .iter()
            .find(|entity| entity.kind == SnapshotKind::Player)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::levels::LEVEL_1;
    use crate::world::World;

    #[test]
    fn snapshot_captures_player_and_enemies() {
        let world = World::new(LEVEL_1).unwrap();
        let snapshot = world.snapshot();
        assert_eq!(snapshot.version, SNAPSHOT_VERSION);
        assert!(snapshot.player().is_some());
        let enemies = snapshot
            .entities
            .iter()
            .filter(|entity| entity.kind == SnapshotKind::Enemy)
            .count();
        assert!(enemies >= 1, "level deve ter inimigos");
    }

    #[test]
    fn player_snapshot_has_default_loadout() {
        let world = World::new(LEVEL_1).unwrap();
        let snapshot = world.snapshot();
        let player = snapshot.player().unwrap().player.as_ref().unwrap();
        assert_eq!(player.loadout, Loadout::player_default());
        assert_eq!(player.slots.len(), ActionSlot::COUNT);
        assert_eq!(player.catalog.len(), SkillId::COUNT);
    }
}
