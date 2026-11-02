//! Cenários de mundo que precisam da API pública do core.

use enzo_core::events::GameEvent;
use enzo_core::input::TickInput;
use enzo_core::levels::LEVEL_1;
use enzo_core::world::{GamePhase, World, FALL_DEATH_MARGIN};

#[test]
fn falling_below_map_counts_as_death() {
    let mut world = World::new(LEVEL_1).expect("fase 1 deve carregar");
    let below_map = world.level().height_px() + FALL_DEATH_MARGIN + 32.0;

    {
        let player = world
            .store_mut()
            .player_mut(0)
            .expect("jogador deve existir");
        player.body.pos.y = below_map;
        player.body.vel.y = 0.0;
    }

    let output = world.step(TickInput::default());

    assert_eq!(world.phase(), GamePhase::Dead);
    assert!(output
        .events
        .iter()
        .any(|event| matches!(event, GameEvent::PlayerDied { .. })));
}

#[test]
fn write_snapshot_reuses_the_entity_buffer() {
    let world = World::new(LEVEL_1).expect("fase 1 deve carregar");
    let expected = world.snapshot();
    let mut buffer = expected.clone();
    let capacity = buffer.entities.capacity();

    world.write_snapshot(&mut buffer);

    assert_eq!(buffer, expected);
    assert_eq!(buffer.entities.capacity(), capacity);
}
