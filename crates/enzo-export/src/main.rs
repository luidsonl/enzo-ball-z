use std::fs;
use std::path::{Path, PathBuf};

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
}
