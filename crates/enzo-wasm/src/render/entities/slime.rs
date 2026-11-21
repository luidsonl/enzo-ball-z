//! Animação e desenho da Slime (lazy modular entity sheet).

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::{self, Sprites};

pub fn frame(entity: &EntitySnapshot, time: f32) -> usize {
    use sprites::slime as f;
    if entity.health <= 0.0 {
        return match ((time * 8.0).floor() as usize) % 2 {
            0 => f::DYING_A,
            _ => f::DYING_B,
        };
    }
    let stun = entity.enemy.as_ref().map(|e| e.stun).unwrap_or(0.0);
    if stun > 0.0 {
        return f::HURT;
    }
    match (time * 7.0) as usize % 3 {
        0 => f::WALK_A,
        1 => f::WALK_B,
        _ => f::STRETCH,
    }
}

pub fn draw(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let Some(_data) = entity.enemy.as_ref() else {
        return;
    };
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let frame = frame(entity, time);
    let is_dead = entity.health <= 0.0;
    let stun = entity.enemy.as_ref().map(|e| e.stun).unwrap_or(0.0);
    let tint = if is_dead {
        Color::from_rgba(235, 235, 245, 220)
    } else if stun > 0.0 {
        Color::from_rgba(255, 210, 210, 255)
    } else {
        WHITE
    };

    draw_ellipse(
        pos.x + size.x * 0.5,
        pos.y + size.y - 2.0,
        size.x * 0.475,
        3.5,
        0.0,
        Color::from_rgba(12, 10, 22, 70),
    );
    let draw_size = (size.x.max(size.y) * 1.25).max(16.0);
    sprites.draw_slime(
        frame,
        Rect::new(
            center.x - draw_size * 0.5,
            pos.y + size.y - draw_size,
            draw_size,
            draw_size,
        ),
        entity.facing < 0,
        tint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use enzo_core::entities::{AnimationState, VisualProfile};
    use enzo_core::math::Vec2;
    use enzo_core::snapshot::{EnemySnapshot, EntitySnapshot, SnapshotKind};

    fn slime_entity(health: f32, stun: f32) -> EntitySnapshot {
        EntitySnapshot {
            id: enzo_core::entities::EntityId(1),
            kind: SnapshotKind::Enemy,
            pos: Vec2::ZERO,
            vel: Vec2::ZERO,
            size: Vec2::new(26.0, 26.0),
            facing: 1,
            on_ground: true,
            dashing: false,
            running: false,
            team: 1,
            health,
            max_health: 3.0,
            iframes: 0.0,
            animation: AnimationState::default(),
            visual_character: None,
            visual_profile: VisualProfile::default(),
            player: None,
            enemy: Some(EnemySnapshot {
                species: enzo_core::entities::EnemySpecies::Slime,
                patrol_dir: 1,
                stun,
            }),
            projectile: None,
            item: None,
        }
    }

    #[test]
    fn slime_uses_hurt_and_dying_frames() {
        let alive = slime_entity(3.0, 0.0);
        assert_eq!(frame(&alive, 0.0), sprites::slime::WALK_A);

        let stunned = slime_entity(3.0, 0.2);
        assert_eq!(frame(&stunned, 0.0), sprites::slime::HURT);

        let dying = slime_entity(0.0, 0.0);
        assert_eq!(frame(&dying, 0.0), sprites::slime::DYING_A);
        assert_eq!(frame(&dying, 0.125), sprites::slime::DYING_B);
    }
}
