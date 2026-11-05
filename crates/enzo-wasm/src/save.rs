//! Persistência local da progressão e das preferências.
//!
//! O core fornece [`SaveData`] versionado; o cliente só grava o JSON no
//! `localStorage` (via `quad-storage`) e o lê de volta. Há [`SLOT_COUNT`] slots
//! independentes. A chave antiga (`enzoballz.save.v1`) continua sendo lida como
//! fallback do slot 0 para não perder o progresso de quem já jogou.
//!
//! Posição, inimigos e projéteis não entram no save.

use enzo_core::progression::{SaveData, SAVE_VERSION};
use quad_storage::STORAGE;

use crate::i18n::Language;

pub const SLOT_COUNT: usize = 3;
pub const LEGACY_KEY: &str = "enzoballz.save.v1";
pub const SLOT_KEY_PREFIX: &str = "enzoballz.save.slot";
pub const LANGUAGE_KEY: &str = "enzoballz.language";

pub fn slot_key(slot: usize) -> String {
    format!("{SLOT_KEY_PREFIX}.{slot}.v1")
}

/// Lê o save de um slot. Versão incompatível ou JSON inválido é ignorado.
pub fn load_slot(slot: usize) -> Option<SaveData> {
    if slot >= SLOT_COUNT {
        return None;
    }

    let Some(raw) = read_key(&slot_key(slot)) else {
        // Migração silenciosa: o save antigo de slot único vira o slot 0.
        return if slot == 0 {
            read_key(LEGACY_KEY).and_then(|raw| parse_save(&raw))
        } else {
            None
        };
    };

    parse_save(&raw)
}

pub fn load_slots() -> [Option<SaveData>; SLOT_COUNT] {
    std::array::from_fn(load_slot)
}

pub fn store_slot(slot: usize, data: &SaveData) -> Result<(), String> {
    if slot >= SLOT_COUNT {
        return Err(format!("slot {slot} inválido"));
    }
    let json = serde_json::to_string(data).map_err(|err| err.to_string())?;
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

fn parse_save(raw: &str) -> Option<SaveData> {
    match serde_json::from_str::<SaveData>(raw) {
        Ok(data) if data.version == SAVE_VERSION => Some(data),
        Ok(data) => {
            macroquad::logging::warn!(
                "save: versão {} incompatível (esperado {})",
                data.version,
                SAVE_VERSION
            );
            None
        }
        Err(err) => {
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
        }
    }

    #[test]
    fn save_json_roundtrip_preserves_progress() {
        let data = sample();
        let json = serde_json::to_string(&data).unwrap();
        let parsed: SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, data);
    }

    #[test]
    fn every_slot_has_a_distinct_key() {
        let keys: Vec<String> = (0..SLOT_COUNT).map(slot_key).collect();
        for (index, key) in keys.iter().enumerate() {
            assert!(key.contains(&format!(".{index}.")));
        }
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert!(load_slot(SLOT_COUNT).is_none());
        assert!(store_slot(SLOT_COUNT, &sample()).is_err());
    }

    #[test]
    fn language_code_roundtrips_through_storage_format() {
        assert_eq!(
            Language::from_code(Language::English.code()),
            Some(Language::English)
        );
        assert_eq!(
            Language::from_code(Language::PtBr.code()),
            Some(Language::PtBr)
        );
    }
}
