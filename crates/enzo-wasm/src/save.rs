//! Persistência local da progressão e das preferências.

use enzo_core::progression::{SaveData, SAVE_VERSION, SAVE_VERSION_V1};
use quad_storage::STORAGE;

use crate::i18n::Language;

pub const SLOT_COUNT: usize = 3;
pub const LEGACY_KEY: &str = "enzoballz.save.v1";
pub const SLOT_KEY_PREFIX: &str = "enzoballz.save.slot";
pub const LANGUAGE_KEY: &str = "enzoballz.language";
pub const INTERFACE_SCALE_KEY: &str = "enzoballz.interface_scale";

pub fn slot_key(slot: usize) -> String {
    format!("{SLOT_KEY_PREFIX}.{slot}.v2")
}

pub fn slot_key_v1(slot: usize) -> String {
    format!("{SLOT_KEY_PREFIX}.{slot}.v1")
}

/// Lê o save de um slot, migrando v1 quando necessário.
pub fn load_slot(slot: usize) -> Option<SaveData> {
    if slot >= SLOT_COUNT {
        return None;
    }

    if let Some(raw) = read_key(&slot_key(slot)) {
        if let Some(data) = parse_save(&raw) {
            return Some(data);
        }
    }

    // Fallback v1 por slot
    if let Some(raw) = read_key(&slot_key_v1(slot)) {
        if let Some(data) = parse_save(&raw) {
            let _ = store_slot(slot, &data);
            return Some(data);
        }
    }

    // Migração silenciosa: o save antigo de slot único vira o slot 0.
    if slot == 0 {
        if let Some(raw) = read_key(LEGACY_KEY) {
            if let Some(data) = parse_save(&raw) {
                let _ = store_slot(0, &data);
                return Some(data);
            }
        }
    }

    None
}

pub fn load_slots() -> [Option<SaveData>; SLOT_COUNT] {
    std::array::from_fn(load_slot)
}

pub fn store_slot(slot: usize, data: &SaveData) -> Result<(), String> {
    if slot >= SLOT_COUNT {
        return Err(format!("slot {slot} inválido"));
    }
    let mut data = *data;
    data.version = SAVE_VERSION;
    let json = serde_json::to_string(&data).map_err(|err| err.to_string())?;
    write_key(&slot_key(slot), &json)
}

pub fn delete_slot(slot: usize) -> Result<(), String> {
    if slot >= SLOT_COUNT {
        return Err(format!("slot {slot} inválido"));
    }
    remove_key(&slot_key(slot))
}

pub fn load_language() -> Language {
    read_key(LANGUAGE_KEY)
        .and_then(|code| Language::from_code(&code))
        .unwrap_or_default()
}

pub fn store_language(language: Language) -> Result<(), String> {
    write_key(LANGUAGE_KEY, language.code())
}

pub fn load_interface_scale() -> f32 {
    read_key(INTERFACE_SCALE_KEY)
        .and_then(|raw| {
            raw.parse::<f32>()
                .ok()
                .filter(|&v| (0.75..=2.0).contains(&v))
        })
        .unwrap_or(1.25)
}

pub fn store_interface_scale(interface_scale: f32) -> Result<(), String> {
    if !(0.75..=2.0).contains(&interface_scale) {
        return Err("escala de interface fora do intervalo válido".to_owned());
    }
    write_key(INTERFACE_SCALE_KEY, &interface_scale.to_string())
}

fn parse_save(raw: &str) -> Option<SaveData> {
    match serde_json::from_str::<SaveData>(raw) {
        Ok(data) => data.migrate().or_else(|| {
            macroquad::logging::warn!("save: versão incompatível");
            None
        }),
        Err(err) => {
            // Tenta interpretar campos v1 sem loadout se o formato antigo falhar por detalhe.
            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(raw) {
                if value.get("version").and_then(|v| v.as_u64()) == Some(SAVE_VERSION_V1 as u64) {
                    if let Ok(data) = serde_json::from_value::<SaveData>(value.take()) {
                        return data.migrate();
                    }
                }
            }
            macroquad::logging::warn!("save: JSON inválido: {err}");
            None
        }
    }
}

fn read_key(key: &str) -> Option<String> {
    let storage = match STORAGE.lock() {
        Ok(storage) => storage,
        Err(_) => {
            macroquad::logging::warn!("save: não foi possível acessar o storage");
            return None;
        }
    };
    storage.get(key)
}

fn write_key(key: &str, value: &str) -> Result<(), String> {
    let mut storage = STORAGE
        .lock()
        .map_err(|_| "storage indisponível".to_string())?;
    storage.set(key, value);
    Ok(())
}

fn remove_key(key: &str) -> Result<(), String> {
    let mut storage = STORAGE
        .lock()
        .map_err(|_| "storage indisponível".to_string())?;
    storage.remove(key);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use enzo_core::entities::Attributes;
    use enzo_core::skills::Loadout;

    fn sample() -> SaveData {
        SaveData {
            version: SAVE_VERSION,
            level: 4,
            xp: 42.0,
            points: 2,
            attributes: Attributes {
                speed: 1,
                strength: 2,
                health: 3,
                aura: 4,
            },
            loadout: Loadout::player_default(),
        }
    }

    #[test]
    fn save_json_roundtrip_preserves_progress() {
        let data = sample();
        let json = serde_json::to_string(&data).unwrap();
        let parsed = parse_save(&json).unwrap();
        assert_eq!(parsed.level, data.level);
        assert_eq!(parsed.loadout, data.loadout);
    }

    #[test]
    fn every_slot_has_a_distinct_key() {
        let keys: Vec<String> = (0..SLOT_COUNT).map(slot_key).collect();
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert!(load_slot(SLOT_COUNT).is_none());
        assert!(store_slot(SLOT_COUNT, &sample()).is_err());
    }
}
