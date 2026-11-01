//! Máquina de estados do farming de aura (`6`/`7`).
//!
//! Alternar as teclas preenche a aura; repetir a mesma tecla zera o combo.
//! Enquanto o combo está ativo o jogador fica lento, então farmar é uma
//! decisão de risco.

use crate::entities::PlayerData;
use crate::input::FarmKey;

pub const AURA_PER_HIT: f32 = 2.0;
pub const COMBO_MULTIPLIER_STEP: f32 = 0.15;
pub const MAX_MULTIPLIER: f32 = 3.0;
pub const COMBO_WINDOW: f32 = 0.9;
pub const FARM_SLOW_FACTOR: f32 = 0.45;

/// Aplica uma tecla de farming. Retorna a aura efetivamente ganha.
pub fn apply_farm(player: &mut PlayerData, key: FarmKey, max_aura: f32) -> f32 {
    if player.farm.last_key == Some(key) {
        // Repetir a tecla quebra a sequência e não gera aura.
        player.farm.combo = 0;
        player.farm.window = 0.0;
        player.farm.last_key = Some(key);
        return 0.0;
    }

    player.farm.combo += 1;
    player.farm.last_key = Some(key);
    player.farm.window = COMBO_WINDOW;

    let multiplier = combo_multiplier(player.farm.combo);
    let gain = (AURA_PER_HIT * multiplier)
        .min(max_aura - player.aura)
        .max(0.0);
    player.aura = (player.aura + gain).min(max_aura);
    player.farm.total += gain;
    gain
}

/// Expira o combo quando a janela termina.
pub fn update(player: &mut PlayerData, dt: f32) {
    if player.farm.window <= 0.0 {
        return;
    }

    player.farm.window = (player.farm.window - dt).max(0.0);
    if player.farm.window <= 0.0 {
        player.farm.combo = 0;
        player.farm.last_key = None;
    }
}

pub fn combo_multiplier(combo: u32) -> f32 {
    (1.0 + COMBO_MULTIPLIER_STEP * combo as f32).min(MAX_MULTIPLIER)
}

/// Combo ativo deixa o jogador lento.
pub fn is_farming(player: &PlayerData) -> bool {
    player.farm.combo > 0 && player.farm.window > 0.0
}

pub fn speed_factor(player: &PlayerData) -> f32 {
    if is_farming(player) {
        FARM_SLOW_FACTOR
    } else {
        1.0
    }
}

/// Gasta aura se houver saldo suficiente. Sem saldo, não gasta nada.
pub fn spend(player: &mut PlayerData, amount: f32) -> bool {
    if player.aura < amount {
        return false;
    }
    player.aura -= amount;
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Stats;
    use crate::math::Vec2;

    fn player() -> PlayerData {
        PlayerData::new(0, Vec2::ZERO, Stats::player(Default::default()))
    }

    fn press(player: &mut PlayerData, key: FarmKey) -> f32 {
        apply_farm(player, key, 100.0)
    }

    #[test]
    fn alternation_increases_combo_and_aura() {
        let mut player = player();
        player.aura = 0.0;
        let first = press(&mut player, FarmKey::Six);
        let second = press(&mut player, FarmKey::Seven);
        let third = press(&mut player, FarmKey::Six);
        assert_eq!(player.farm.combo, 3);
        assert!(first > 0.0);
        assert!(second > first);
        assert!(third > second);
        assert!((first - 2.3).abs() < 0.001);
        assert!((second - 2.6).abs() < 0.001);
        assert!((third - 2.9).abs() < 0.001);
    }

    #[test]
    fn repeated_key_breaks_combo_and_gives_nothing() {
        let mut player = player();
        player.aura = 0.0;
        press(&mut player, FarmKey::Six);
        let gain = press(&mut player, FarmKey::Six);
        assert_eq!(gain, 0.0);
        assert_eq!(player.farm.combo, 0);
        assert_eq!(player.farm.window, 0.0);
        // Depois da quebra, a próxima tecla reinicia o combo.
        press(&mut player, FarmKey::Seven);
        assert_eq!(player.farm.combo, 1);
    }

    #[test]
    fn multiplier_is_capped() {
        assert_eq!(combo_multiplier(0), 1.0);
        assert_eq!(combo_multiplier(10), 2.5);
        assert_eq!(combo_multiplier(100), MAX_MULTIPLIER);
    }

    #[test]
    fn aura_never_exceeds_max() {
        let mut player = player();
        player.aura = 99.0;
        press(&mut player, FarmKey::Six);
        assert_eq!(player.aura, 100.0);
    }

    #[test]
    fn combo_expires_and_clears_last_key() {
        let mut player = player();
        player.aura = 0.0;
        press(&mut player, FarmKey::Six);
        assert!(is_farming(&player));
        for _ in 0..60 {
            update(&mut player, crate::DT);
        }
        assert!(!is_farming(&player));
        assert_eq!(player.farm.combo, 0);
        assert_eq!(player.farm.last_key, None);
        // Após expirar, a mesma tecla conta como novo começo.
        let gain = press(&mut player, FarmKey::Six);
        assert!(gain > 0.0);
    }

    #[test]
    fn farming_slows_movement() {
        let mut player = player();
        assert_eq!(speed_factor(&player), 1.0);
        press(&mut player, FarmKey::Six);
        assert_eq!(speed_factor(&player), FARM_SLOW_FACTOR);
    }

    #[test]
    fn spend_requires_full_cost() {
        let mut player = player();
        player.aura = 20.0;
        assert!(!spend(&mut player, 25.0));
        assert_eq!(player.aura, 20.0);
        player.aura = 30.0;
        assert!(spend(&mut player, 25.0));
        assert_eq!(player.aura, 5.0);
    }
}
