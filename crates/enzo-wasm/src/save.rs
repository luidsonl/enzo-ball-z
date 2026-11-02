//! Persistência local da progressão.
//!
//! O core fornece [`SaveData`] versionado; o cliente só grava o JSON no
//! `localStorage` (via `quad-storage`) e o lê de volta. Posição, inimigos e
//! projéteis não entram no save.

use enzo_core::progression::{SaveData, SAVE_VERSION};
use quad_storage::STORAGE;

pub const KEY: &str = "enzoballz.save.v1";

/// Lê o save. Versão incompatível ou JSON inválido é ignorado com aviso.
pub fn load() -> Option<SaveData> {
    let storage = match STORAGE.lock() {
        Ok(storage) => storage,
        Err(_) => {
            macroquad::logging::warn!("save: não foi possível acessar o storage");
            return None;
        }
    };
    let raw = storage.get(KEY)?;

    match serde_json::from_str::<SaveData>(&raw) {
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

pub fn store(data: &SaveData) -> Result<(), String> {
    let json = serde_json::to_string(data).map_err(|err| err.to_string())?;
    let mut storage = STORAGE
        .lock()
        .map_err(|_| "storage indisponível".to_string())?;
    storage.set(KEY, &json);
    Ok(())
}

pub fn clear() -> Result<(), String> {
    let mut storage = STORAGE
        .lock()
        .map_err(|_| "storage indisponível".to_string())?;
    storage.remove(KEY);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use enzo_core::entities::Attributes;

    #[test]
    fn save_json_roundtrip_preserves_progress() {
        let data = SaveData {
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
        };

        let json = serde_json::to_string(&data).unwrap();
        let parsed: SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, data);
    }
}
