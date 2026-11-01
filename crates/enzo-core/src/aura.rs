//! Recurso de aura.
//!
//! O ganho por `6`/`7` é decidido pelo movimento
//! [`crate::movements::SixSeven`]; aqui ficam só as operações sobre o saldo do
//! jogador: aplicar o ganho do movimento e gastar no Mogar.

use crate::entities::PlayerData;
use crate::input::SixSevenKey;
use crate::movements::{sixseven, SixSeven};

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

    let gain = (sixseven::AURA_PER_HIT * multiplier)
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
        PlayerData::new(0, Vec2::ZERO, Stats::player(Default::default()))
    }

    fn press(player: &mut PlayerData, sixseven: &mut SixSeven, key: SixSevenKey) -> f32 {
        apply_sixseven(player, sixseven, key, 100.0)
    }

    #[test]
    fn alternation_increases_combo_and_aura() {
        let mut player = player();
        let mut sixseven = SixSeven::default();
        player.aura = 0.0;
        let first = press(&mut player, &mut sixseven, SixSevenKey::Six);
        let second = press(&mut player, &mut sixseven, SixSevenKey::Seven);
        let third = press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert_eq!(sixseven.combo, 3);
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
        player.aura = 99.0;
        press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert_eq!(player.aura, 100.0);
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
        // Após expirar, a mesma tecla conta como novo começo.
        let gain = press(&mut player, &mut sixseven, SixSevenKey::Six);
        assert!(gain > 0.0);
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
