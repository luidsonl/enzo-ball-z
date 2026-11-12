//! Recurso de aura.
//!
//! O ganho por `6`/`7` é decidido pela mecânica SixSeven; aqui ficam só as
//! operações sobre o saldo do jogador.

use crate::entities::PlayerData;
use crate::input::SixSevenKey;
use crate::mechanics::{sixseven, SixSeven};

/// Aplica a tecla de SixSeven ao saldo de aura do jogador. Retorna a aura
/// efetivamente ganha.
pub fn apply_sixseven(
    player: &mut PlayerData,
    sixseven: &mut SixSeven,
    key: SixSevenKey,
    max_aura: f32,
) -> f32 {
    let multiplier = sixseven.press(key);
    if multiplier <= 0.0 {
        return 0.0;
    }

    let gain = sixseven::aura_gain_for_combo(sixseven.combo, max_aura)
        .min(max_aura - player.aura)
        .max(0.0);
    player.aura = (player.aura + gain).min(max_aura);
    sixseven.record_gain(gain);
    gain
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
        use crate::entities::{PlayableCharacter, VisualProfile};
        PlayerData::new(
            0,
            PlayableCharacter::Enzo,
            VisualProfile::default(),
            Vec2::ZERO,
            Stats::player(Default::default()),
        )
    }

    fn press(player: &mut PlayerData, sixseven: &mut SixSeven, key: SixSevenKey) -> f32 {
        apply_sixseven(player, sixseven, key, 10.0)
    }

    #[test]
    fn alternation_increases_combo_and_aura() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        let first = press(&mut player, &mut sixseven, SixSevenKey::Six);
        let second = press(&mut player, &mut sixseven, SixSevenKey::Seven);
        let third = press(&mut player, &mut sixseven, SixSevenKey::Six);
        let fourth = press(&mut player, &mut sixseven, SixSevenKey::Seven);
        assert_eq!(sixseven.combo, 4);
        assert_eq!(first, 0.0);
        assert_eq!(second, 0.0);
        assert_eq!(third, 0.0);
        assert_eq!(fourth, 1.0); // 10 aura over 40 hits => 1 every 4 alternations
    }

    #[test]
    fn forty_alternations_fill_base_aura() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        let target_hits = sixseven::aura_fill_hits_for_max(10.0);
        assert_eq!(target_hits, 40);
        let mut key = SixSevenKey::Six;
        for i in 0..target_hits {
            press(&mut player, &mut sixseven, key);
            key = match key {
                SixSevenKey::Six => SixSevenKey::Seven,
                SixSevenKey::Seven => SixSevenKey::Six,
            };
            if i < target_hits - 1 {
                assert!(player.aura < 10.0, "aura filled too early at hit {}", i + 1);
            }
        }
        assert_eq!(sixseven.combo, 40);
        assert_eq!(player.aura, 10.0);
    }

    #[test]
    fn fill_time_scales_with_total_aura() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        let max_aura = 20.0; // 2 alternations per aura point
        let target_hits = sixseven::aura_fill_hits_for_max(max_aura);
        assert_eq!(target_hits, 40);
        let mut key = SixSevenKey::Six;
        for i in 0..target_hits {
            apply_sixseven(&mut player, &mut sixseven, key, max_aura);
            key = match key {
                SixSevenKey::Six => SixSevenKey::Seven,
                SixSevenKey::Seven => SixSevenKey::Six,
            };
            if i < target_hits - 1 {
                assert!(
                    player.aura < max_aura,
                    "aura filled too early at hit {}",
                    i + 1
                );
            }
        }
        assert_eq!(player.aura, max_aura);
    }

    #[test]
    fn high_aura_grants_multiple_per_alternation() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        let max_aura = 80.0;
        let first = apply_sixseven(&mut player, &mut sixseven, SixSevenKey::Six, max_aura);
        assert_eq!(first, 2.0); // 80 aura / 40 hits
        let mut key = SixSevenKey::Seven;
        for _ in 1..40 {
            apply_sixseven(&mut player, &mut sixseven, key, max_aura);
            key = match key {
                SixSevenKey::Six => SixSevenKey::Seven,
                SixSevenKey::Seven => SixSevenKey::Six,
            };
        }
        assert_eq!(player.aura, max_aura);
    }

    #[test]
    fn repeated_key_breaks_combo_and_gives_nothing() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        press(&mut player, &mut sixseven, SixSevenKey::Six);
        let gain = press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert_eq!(gain, 0.0);
        assert_eq!(sixseven.combo, 0);
        assert_eq!(sixseven.window, 0.0);
        // Depois da quebra, a próxima tecla reinicia o combo.
        press(&mut player, &mut sixseven, SixSevenKey::Seven);
        assert_eq!(sixseven.combo, 1);
    }

    #[test]
    fn aura_never_exceeds_max() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 9.0;
        press(&mut player, &mut sixseven, SixSevenKey::Six);
        press(&mut player, &mut sixseven, SixSevenKey::Seven);
        press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert_eq!(player.aura, 9.0);
        press(&mut player, &mut sixseven, SixSevenKey::Seven); // 4th hit => +1, capped
        assert_eq!(player.aura, 10.0);
    }

    #[test]
    fn combo_expires_and_clears_last_key() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert!(sixseven.is_active());
        for _ in 0..60 {
            sixseven.tick(crate::DT);
        }
        assert!(!sixseven.is_active());
        assert_eq!(sixseven.combo, 0);
        assert_eq!(sixseven.last_key, None);
        // Após expirar, a mesma tecla conta como novo começo (combo reinicia em 1).
        let gain = press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert_eq!(sixseven.combo, 1);
        assert_eq!(gain, 0.0);
    }

    #[test]
    fn spend_requires_full_cost() {
        let mut player = player();
        player.aura = 2.0;
        assert!(!spend(&mut player, 3.0));
        assert_eq!(player.aura, 2.0);
        player.aura = 5.0;
        assert!(spend(&mut player, 3.0));
        assert_eq!(player.aura, 2.0);
    }
}
