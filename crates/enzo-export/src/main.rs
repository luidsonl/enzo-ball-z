use std::fs;
use std::path::{Path, PathBuf};

use enzo_core::entities::{AttributeId, ItemKind};
use enzo_core::map::{block_def, BlockId};
use enzo_core::skills::SkillId;
use serde::Serialize;

#[derive(Serialize)]
struct BlockOut {
    id: String,
    name: &'static str,
    symbol: Option<char>,
    width: u8,
    height: u8,
    collision: String,
    resistance: serde_json::Value,
    special: String,
    render: String,
}

#[derive(Serialize)]
struct SkillOut {
    id: String,
    category: String,
    source: String,
    activation: String,
    aura_cost: f32,
    cooldown: f32,
    requirements: serde_json::Value,
    assets: serde_json::Value,
}

fn blocks() -> Vec<BlockOut> {
    let ids = [
        BlockId::Empty,
        BlockId::Ground,
        BlockId::Block,
        BlockId::Platform,
        BlockId::Crate,
    ];
    ids.into_iter()
        .map(|id| {
            let d = block_def(id);
            BlockOut {
                id: format!("{id:?}").to_lowercase(),
                name: d.name,
                symbol: d.symbol,
                width: d.width,
                height: d.height,
                collision: format!("{:?}", d.collision).to_lowercase(),
                resistance: serde_json::json!(format!("{:?}", d.resistance)),
                special: format!("{:?}", d.special).to_lowercase(),
                render: format!("{:?}", d.render).to_lowercase(),
            }
        })
        .collect()
}

fn skills() -> Vec<SkillOut> {
    SkillId::ALL
        .into_iter()
        .map(|id| {
            let d = id.def();
            SkillOut {
                id: format!("{id:?}"),
                category: format!("{:?}", d.category).to_lowercase(),
                source: format!("{:?}", d.source).to_lowercase(),
                activation: format!("{:?}", d.activation).to_lowercase(),
                aura_cost: d.aura_cost,
                cooldown: d.cooldown,
                requirements: serde_json::json!({
                    "min_level": d.requirements.min_level,
                    "entity_types": format!("{:?}", d.requirements.entity_types),
                }),
                assets: serde_json::json!({
                    "icon": d.assets.icon,
                    "entity": d.assets.entity,
                    "particles": d.assets.particles,
                }),
            }
        })
        .collect()
}

fn write_json<T: Serialize>(path: &Path, value: &T) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let mut text = serde_json::to_string_pretty(value).unwrap();
    text.push('\n');
    fs::write(path, text).unwrap();
    println!("wrote {}", path.display());
}

fn attr_id_name(id: AttributeId) -> &'static str {
    match id {
        AttributeId::SPEED => "speed",
        AttributeId::STRENGTH => "strength",
        AttributeId::HEALTH => "health",
        AttributeId::AURA => "aura",
        AttributeId::EGO => "ego",
        AttributeId::MELEE => "melee",
        AttributeId::RANGED => "ranged",
        _ => "unknown",
    }
}

fn attr_set_json(set: enzo_core::entities::AttributeSet) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    for id in AttributeId::ALL {
        let v = set.get(id);
        if v != 0 {
            obj.insert(attr_id_name(id).to_owned(), serde_json::json!(v));
        }
    }
    serde_json::Value::Object(obj)
}

#[derive(Serialize)]
struct ItemOut {
    id: String,
    kind: String,
    category: String,
    slot: Option<String>,
    usage: String,
    consumable: bool,
    heal: f32,
    equipment_tags: Vec<String>,
    equipment_modifiers: serde_json::Value,
    requirements: serde_json::Value,
    visual: serde_json::Value,
}

fn items() -> Vec<ItemOut> {
    ItemKind::ALL
        .into_iter()
        .map(|kind| {
            let d = kind.data();
            let mut tags = Vec::new();
            let et = d.equipment.tags;
            if et.contains(enzo_core::entities::EquipmentTags::SWORD) {
                tags.push("sword".to_owned());
            }
            if et.contains(enzo_core::entities::EquipmentTags::BOW) {
                tags.push("bow".to_owned());
            }
            if et.contains(enzo_core::entities::EquipmentTags::NUTELLA_GUN) {
                tags.push("nutella_gun".to_owned());
            }
            if et.contains(enzo_core::entities::EquipmentTags::SIXSEVEN) {
                tags.push("sixseven".to_owned());
            }
            ItemOut {
                id: kind.id().to_owned(),
                kind: format!("{kind:?}"),
                category: format!("{:?}", d.category).to_lowercase(),
                slot: d.slot().map(|s| format!("{s:?}").to_lowercase()),
                usage: format!("{:?}", d.usage()).to_lowercase(),
                consumable: d.is_consumable(),
                heal: d.heal,
                equipment_tags: tags,
                equipment_modifiers: attr_set_json(d.equipment.modifiers),
                requirements: serde_json::json!({
                    "entity_types": format!("{:?}", d.requirements.entity_types),
                    "min_attributes": attr_set_json(d.requirements.min_attributes),
                }),
                visual: serde_json::json!({
                    "icon": format!("assets/items/{}/icon.png", kind.id()),
                    "world": format!("assets/items/{}/world.png", kind.id()),
                    "equipped": format!("assets/items/{}/equipped.png", kind.id()),
                }),
            }
        })
        .collect()
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = root.join("assets/definitions/generated");
    write_json(
        &out.join("blocks.json"),
        &serde_json::json!({"source": "enzo-core", "blocks": blocks()}),
    );
    write_json(
        &out.join("skills.json"),
        &serde_json::json!({"source": "enzo-core", "skills": skills()}),
    );
    write_json(
        &out.join("items.json"),
        &serde_json::json!({"source": "enzo-core", "items": items()}),
    );
}
