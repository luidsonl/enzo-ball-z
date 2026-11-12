//! Movimento **SixSeven**: alternância `6`/`7` para farmar aura.
//!
//! Alternar as teclas preenche a aura; repetir a mesma tecla zera o combo.
//! Enquanto o combo está ativo o jogador fica lento, então usar SixSeven é uma
//! decisão de risco.
//!
//! O componente guarda apenas o estado do combo. A aura em si é recurso do
//! jogador e continua em [`crate::entities::PlayerData`]; o sistema de aura
//! aplica o ganho retornado por [`SixSeven::press`].

use crate::entities::{AttributeSet, EquipmentTags};
use crate::input::SixSevenKey;
use crate::skills::{EntityType, Requirements};

pub const AURA_PER_HIT: f32 = 1.0;
pub const COMBO_MULTIPLIER_STEP: f32 = 0.15;
pub const MAX_MULTIPLIER: f32 = 3.0;
pub const COMBO_WINDOW: f32 = 0.9;
pub const SLOW_FACTOR: f32 = 0.45;
/// Successful alternations needed to fully fill the aura bar.
/// Tuned so 10 max aura takes ~4 alternations per point, 20 takes 2, 40 takes 1,
/// and 80 grants 2 aura per alternation (constant 40 alternations to fill).
pub const AURA_FILL_HITS: u32 = 40;

/// Target alternations to fill `max_aura`.
pub fn aura_fill_hits_for_max(_max_aura: f32) -> u32 {
    AURA_FILL_HITS
}

/// SixSeven é exclusivo de jogáveis no MVP.
pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

/// Estado do SixSeven anexado ao componente [`crate::movements::Movement`].
///
/// `pending` é o edge recebido do input neste tick. O sistema de aura o consome
/// antes de o movimento avançar o tempo, para o ganho acontecer no mesmo tick
/// da tecla.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SixSeven {
    pub last_key: Option<SixSevenKey>,
    pub combo: u32,
    pub window: f32,
    pub total: f32,
    pending: Option<SixSevenKey>,
}

impl SixSeven {
    pub const fn new() -> Self {
        Self {
            last_key: None,
            combo: 0,
            window: 0.0,
            total: 0.0,
            pending: None,
        }
    }

    /// Registra a tecla pressionada. A aplicação na aura acontece no sistema de
    /// aura, não no input.
    pub fn request(&mut self, key: SixSevenKey) {
        self.pending = Some(key);
    }

    pub fn take_request(&mut self) -> Option<SixSevenKey> {
        self.pending.take()
    }

    /// Expira o combo quando a janela termina.
    pub fn tick(&mut self, dt: f32) {
        if self.window <= 0.0 {
            return;
        }

        self.window = (self.window - dt).max(0.0);
        if self.window <= 0.0 {
            self.combo = 0;
            self.last_key = None;
        }
    }

    /// Aplica uma tecla ao combo e devolve o multiplicador do ganho. Retorna
    /// zero quando a tecla repetida quebra a sequência.
    pub fn press(&mut self, key: SixSevenKey) -> f32 {
        if self.last_key == Some(key) {
            self.combo = 0;
            self.window = 0.0;
            self.last_key = Some(key);
            return 0.0;
        }

        self.combo += 1;
        self.last_key = Some(key);
        self.window = COMBO_WINDOW;
        combo_multiplier(self.combo)
    }

    pub fn record_gain(&mut self, gain: f32) {
        self.total += gain;
    }

    /// Combo ativo deixa o jogador lento.
    pub fn is_active(&self) -> bool {
        self.combo > 0 && self.window > 0.0
    }

    pub fn speed_factor(&self) -> f32 {
        if self.is_active() {
            SLOW_FACTOR
        } else {
            1.0
        }
    }
}

pub fn combo_multiplier(combo: u32) -> f32 {
    (1.0 + COMBO_MULTIPLIER_STEP * combo as f32).min(MAX_MULTIPLIER)
}

/// Integer aura gain for a successful alternation at `combo` (1-based within streak).
/// Distributed deterministically so filling `max_aura` takes [`AURA_FILL_HITS`]
/// alternations. This yields fractional-per-point pacing (e.g. 1 aura every 2-4 hits
/// at low max aura) and multiple aura per hit at high max aura.
pub fn aura_gain_for_combo(combo: u32, max_aura: f32) -> f32 {
    if combo == 0 {
        return 0.0;
    }
    let total = max_aura.max(0.0).round() as i32;
    let hits = AURA_FILL_HITS as i32;
    if total <= 0 {
        return 0.0;
    }
    let c = combo as i32;
    let prev = ((c - 1) * total) / hits;
    let curr = (c * total) / hits;
    (curr - prev).max(0) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alternation_increases_combo_and_multiplier() {
        let mut sixseven = SixSeven::default();
        let first = sixseven.press(SixSevenKey::Six);
        let second = sixseven.press(SixSevenKey::Seven);
        let third = sixseven.press(SixSevenKey::Six);
        assert_eq!(sixseven.combo, 3);
        assert!((first - 1.15).abs() < 0.001);
        assert!((second - 1.30).abs() < 0.001);
        assert!((third - 1.45).abs() < 0.001);
    }

    #[test]
    fn repeated_key_breaks_combo() {
        let mut sixseven = SixSeven::default();
        sixseven.press(SixSevenKey::Six);
        let gain = sixseven.press(SixSevenKey::Six);
        assert_eq!(gain, 0.0);
        assert_eq!(sixseven.combo, 0);
        assert_eq!(sixseven.window, 0.0);
        sixseven.press(SixSevenKey::Seven);
        assert_eq!(sixseven.combo, 1);
    }

    #[test]
    fn multiplier_is_capped() {
        assert_eq!(combo_multiplier(0), 1.0);
        assert_eq!(combo_multiplier(10), 2.5);
        assert_eq!(combo_multiplier(100), MAX_MULTIPLIER);
    }

    #[test]
    fn combo_expires_and_clears_last_key() {
        let mut sixseven = SixSeven::default();
        sixseven.press(SixSevenKey::Six);
        assert!(sixseven.is_active());
        for _ in 0..60 {
            sixseven.tick(crate::DT);
        }
        assert!(!sixseven.is_active());
        assert_eq!(sixseven.combo, 0);
        assert_eq!(sixseven.last_key, None);
        assert!(sixseven.press(SixSevenKey::Six) > 0.0);
    }

    #[test]
    fn active_combo_slows_movement() {
        let mut sixseven = SixSeven::default();
        assert_eq!(sixseven.speed_factor(), 1.0);
        sixseven.press(SixSevenKey::Six);
        assert_eq!(sixseven.speed_factor(), SLOW_FACTOR);
    }

    #[test]
    fn request_is_consumed_once() {
        let mut sixseven = SixSeven::default();
        sixseven.request(SixSevenKey::Seven);
        assert_eq!(sixseven.take_request(), Some(SixSevenKey::Seven));
        assert_eq!(sixseven.take_request(), None);
    }
}
