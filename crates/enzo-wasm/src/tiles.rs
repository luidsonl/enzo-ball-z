//! Chunked tile cache with lazy tilesheet loading.
//!
//! Instead of drawing every map tile each frame, each visible chunk becomes a
//! static mesh built once. Chunks outside the camera are never built.
//! The tilesheet texture is requested lazily; chunks render once it's ready.

use enzo_core::map::{Level, RenderKind, TILE_SIZE};
use macroquad::prelude::*;

use crate::assets::AssetStore;
use crate::sprites::{self, TILES_SHEET_PATH, TILE_CELL};

/// Chunk side in tiles. 16x16 covers 512x512 px of world space.
pub const CHUNK_TILES: i32 = 16;
const CHUNK_PX: f32 = CHUNK_TILES as f32 * TILE_SIZE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ChunkRange {
    pub min_x: usize,
    pub max_x: usize,
    pub min_y: usize,
    pub max_y: usize,
}

/// Chunks touching the camera, with a one-chunk margin to avoid pop-in.
/// Returns `None` for empty maps.
pub fn visible_chunks(view: Rect, cols: usize, rows: usize) -> Option<ChunkRange> {
    if cols == 0 || rows == 0 {
        return None;
    }

    let last_col = cols as i32 - 1;
    let last_row = rows as i32 - 1;
    let min_x = ((view.x / CHUNK_PX).floor() as i32 - 1).clamp(0, last_col);
    let max_x = (((view.x + view.w) / CHUNK_PX).ceil() as i32 + 1).clamp(0, last_col);
    let min_y = ((view.y / CHUNK_PX).floor() as i32 - 1).clamp(0, last_row);
    let max_y = (((view.y + view.h) / CHUNK_PX).ceil() as i32 + 1).clamp(0, last_row);

    Some(ChunkRange {
        min_x: min_x as usize,
        max_x: max_x as usize,
        min_y: min_y as usize,
        max_y: max_y as usize,
    })
}

pub struct TileRenderer {
    chunks: Vec<Option<Mesh>>,
    cols: usize,
    rows: usize,
    assets: AssetStore,
    built: usize,
}

impl TileRenderer {
    pub fn new(level: &Level, assets: AssetStore) -> Self {
        let cols = level.width().div_ceil(CHUNK_TILES as usize);
        let rows = level.height().div_ceil(CHUNK_TILES as usize);
        assets.request(TILES_SHEET_PATH.to_owned());
        Self {
            chunks: (0..cols * rows).map(|_| None).collect(),
            cols,
            rows,
            assets,
            built: 0,
        }
    }

    /// Builds and draws chunks touching the camera.
    ///
    /// Chunks are rebuilt when drawn so destructible/dynamic block states remain
    /// visible. If the tilesheet isn't loaded yet, triggers lazy load and skips.
    pub fn draw(&mut self, level: &Level, view: Rect) {
        let Some(range) = visible_chunks(view, self.cols, self.rows) else {
            return;
        };
        let Some(texture) = self.assets.get_or_request(TILES_SHEET_PATH) else {
            return;
        };
        let atlas = texture.size();

        for cy in range.min_y..=range.max_y {
            for cx in range.min_x..=range.max_x {
                let index = cy * self.cols + cx;
                let mesh = build_chunk(level, cx, cy, &texture, atlas);
                self.built = self.built.max(index + 1);
                self.chunks[index] = Some(mesh);
                if let Some(mesh) = &self.chunks[index] {
                    if !mesh.vertices.is_empty() {
                        draw_mesh(mesh);
                    }
                }
            }
        }
    }

    pub fn built_chunks(&self) -> usize {
        self.built
    }

    pub fn total_chunks(&self) -> usize {
        self.chunks.len()
    }
}

fn build_chunk(level: &Level, cx: usize, cy: usize, texture: &Texture2D, atlas: Vec2) -> Mesh {
    let start_x = cx as i32 * CHUNK_TILES;
    let start_y = cy as i32 * CHUNK_TILES;
    let end_x = (start_x + CHUNK_TILES).min(level.width() as i32);
    let end_y = (start_y + CHUNK_TILES).min(level.height() as i32);

    let mut vertices = Vec::with_capacity((CHUNK_TILES * CHUNK_TILES * 4) as usize);
    let mut indices = Vec::with_capacity((CHUNK_TILES * CHUNK_TILES * 6) as usize);

    for ty in start_y..end_y {
        for tx in start_x..end_x {
            let tile = level.tile_at(tx, ty);
            let frame = match tile.render_kind() {
                RenderKind::Empty => continue,
                RenderKind::Ground => {
                    if level.tile_at(tx, ty - 1).is_solid() {
                        sprites::tile::GROUND_FILL
                    } else {
                        sprites::tile::GROUND_TOP
                    }
                }
                RenderKind::Block => sprites::tile::BLOCK,
                RenderKind::Platform => sprites::tile::PLATFORM,
                RenderKind::Crate => {
                    let damage = if tx >= 0 && ty >= 0 {
                        level.block_damage(tx as usize, ty as usize)
                    } else {
                        0
                    };
                    match damage {
                        0 => sprites::tile::CRATE,
                        1 => sprites::tile::CRATE_DAMAGED_1,
                        _ => sprites::tile::CRATE_DAMAGED_2,
                    }
                }
                RenderKind::FlagBottom => sprites::tile::FLAG_BOTTOM,
                RenderKind::FlagTop => sprites::tile::FLAG_TOP,
                RenderKind::BreakableStone => {
                    let damage = if tx >= 0 && ty >= 0 {
                        level.block_damage(tx as usize, ty as usize)
                    } else {
                        0
                    };
                    match damage {
                        0 => sprites::tile::BREAKABLE_STONE,
                        1 => sprites::tile::BREAKABLE_STONE_DAMAGED_1,
                        _ => sprites::tile::BREAKABLE_STONE_DAMAGED_2,
                    }
                }
                // Placeholders until dedicated sprites are generated via pixel-forge.
                RenderKind::Water => sprites::tile::PLATFORM,
                RenderKind::Ladder => sprites::tile::BLOCK,
            };
            push_tile(&mut vertices, &mut indices, tx, ty, frame, atlas);
        }
    }

    Mesh {
        vertices,
        indices,
        texture: Some(texture.clone()),
    }
}

fn push_tile(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    tx: i32,
    ty: i32,
    frame: usize,
    atlas: Vec2,
) {
    let x = tx as f32 * TILE_SIZE;
    let y = ty as f32 * TILE_SIZE;
    let sx = frame as f32 * TILE_CELL;
    let u0 = sx / atlas.x;
    let u1 = (sx + TILE_CELL) / atlas.x;
    let v0 = 0.0;
    let v1 = TILE_CELL / atlas.y;
    let base = vertices.len() as u16;

    vertices.extend([
        Vertex::new(x, y, 0.0, u0, v0, WHITE),
        Vertex::new(x + TILE_SIZE, y, 0.0, u1, v0, WHITE),
        Vertex::new(x + TILE_SIZE, y + TILE_SIZE, 0.0, u1, v1, WHITE),
        Vertex::new(x, y + TILE_SIZE, 0.0, u0, v1, WHITE),
    ]);
    indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_chunks_include_a_margin_and_clamp_to_map() {
        let range = visible_chunks(Rect::new(0.0, 0.0, 640.0, 360.0), 7, 2).unwrap();
        assert_eq!(
            range,
            ChunkRange {
                min_x: 0,
                max_x: 3,
                min_y: 0,
                max_y: 1
            }
        );

        let range = visible_chunks(Rect::new(1500.0, 100.0, 640.0, 360.0), 7, 2).unwrap();
        assert_eq!(
            range,
            ChunkRange {
                min_x: 1,
                max_x: 6,
                min_y: 0,
                max_y: 1
            }
        );
    }

    #[test]
    fn empty_maps_have_no_visible_chunks() {
        assert_eq!(visible_chunks(Rect::new(0.0, 0.0, 10.0, 10.0), 0, 0), None);
    }
}
