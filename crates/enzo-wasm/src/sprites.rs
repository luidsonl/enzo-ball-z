//! Original sprites loaded at runtime.
//!
//! PNGs are generated/versioned under `assets/sprites/` and copied to
//! `dist/assets/sprites/` by the build pipeline. The client loads them at
//! runtime so the wasm binary stays smaller and the same image files can be
//! reused outside the wasm (web UI, tooling, future clients).
//!
//! Each sheet uses a fixed cell: tiles and items are 16x16; actors and effects
//! are 32x32. Rendering chooses frames from snapshot/state.

use macroquad::prelude::*;

const TILES_PATH: &str = "assets/sprites/tiles.png";
const ENZO_PATH: &str = "assets/sprites/enzo.png";
const VALENTINA_PATH: &str = "assets/entities/valentina/sheet.png";
const SLIME_PATH: &str = "assets/sprites/slime.png";
const FX_PATH: &str = "assets/sprites/fx.png";
const FLAG_PATH: &str = "assets/sprites/flag.png";
const ITEMS_PATH: &str = "assets/sprites/items.png";

pub const TILE_CELL: f32 = 16.0;
pub const ACTOR_CELL: f32 = 32.0;
pub const ITEM_CELL: f32 = 16.0;

/// Tile sheet frame indices.
pub mod tile {
    pub const GROUND_TOP: usize = 0;
    pub const GROUND_FILL: usize = 1;
    pub const BLOCK: usize = 2;
    pub const PLATFORM: usize = 3;
    pub const CRATE: usize = 4;
    pub const CRATE_DAMAGED_1: usize = 5;
    pub const CRATE_DAMAGED_2: usize = 6;
}

/// Enzo sheet frame indices.
pub mod enzo {
    pub const IDLE_A: usize = 0;
    pub const IDLE_B: usize = 1;
    pub const RUN_A: usize = 2;
    pub const RUN_B: usize = 3;
    pub const RUN_C: usize = 4;
    pub const RUN_D: usize = 5;
    pub const JUMP: usize = 6;
    pub const FALL: usize = 7;
    pub const DASH: usize = 8;
    pub const HURT: usize = 9;
    pub const ATTACK_A: usize = 10;
    pub const ATTACK_B: usize = 11;
    pub const FARM_A: usize = 12;
    pub const FARM_B: usize = 13;
}

/// Valentina sheet frame indices (assets/entities/valentina/sheet.png).
pub mod valentina {
    pub const IDLE: usize = 0;
    pub const WALK_1: usize = 1;
    pub const WALK_2: usize = 2;
    pub const JUMP: usize = 3;
    pub const KICK: usize = 4;
    pub const CAST: usize = 5;
    pub const SIXSEVEN_A: usize = 6;
    pub const SIXSEVEN_B: usize = 7;
    pub const HURT: usize = 8;
}

/// Enemy (slime/minion) sheet frame indices.
pub mod slime {
    pub const WALK_A: usize = 0;
    pub const WALK_B: usize = 1;
    pub const STRETCH: usize = 2;
    pub const HURT: usize = 3;
}

/// Effects sheet frame indices.
pub mod fx {
    pub const MOGGING: usize = 0;
    pub const SLASH_A: usize = 1;
    pub const SLASH_B: usize = 2;
    pub const SLASH_C: usize = 3;
    pub const SPARK_YELLOW: usize = 4;
    pub const SPARK_CYAN: usize = 5;
    pub const SPARK_RED: usize = 6;
    pub const DUST: usize = 7;
}

/// Item sheet frame indices.
pub mod item {
    pub const SWORD: usize = 0;
    pub const BOW: usize = 1;
    pub const ARMOR: usize = 2;
    pub const AMULET: usize = 3;
    pub const POTION: usize = 4;
}

pub struct Sprites {
    tiles: Texture2D,
    enzo: Texture2D,
    valentina: Texture2D,
    slime: Texture2D,
    fx: Texture2D,
    flag: Texture2D,
    items: Texture2D,
}

impl Sprites {
    pub async fn load() -> Self {
        Self {
            tiles: texture(TILES_PATH, "tiles").await,
            enzo: texture(ENZO_PATH, "enzo").await,
            valentina: texture(VALENTINA_PATH, "valentina").await,
            slime: texture(SLIME_PATH, "slime").await,
            fx: texture(FX_PATH, "fx").await,
            flag: texture(FLAG_PATH, "flag").await,
            items: texture(ITEMS_PATH, "items").await,
        }
    }

    pub fn draw_tile(&self, index: usize, dest: Rect, tint: Color) {
        draw_sheet(&self.tiles, TILE_CELL, index, dest, false, tint);
    }

    /// Texture used by the tile chunk cache.
    pub fn tiles_texture(&self) -> &Texture2D {
        &self.tiles
    }

    pub fn draw_enzo(&self, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
        draw_sheet(&self.enzo, ACTOR_CELL, frame, dest, flip_x, tint);
    }

    pub fn draw_valentina(&self, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
        draw_sheet(&self.valentina, ACTOR_CELL, frame, dest, flip_x, tint);
    }

    pub fn draw_slime(&self, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
        draw_sheet(&self.slime, ACTOR_CELL, frame, dest, flip_x, tint);
    }

    pub fn draw_fx(&self, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
        draw_sheet(&self.fx, ACTOR_CELL, frame, dest, flip_x, tint);
    }

    pub fn draw_flag(&self, dest: Rect, tint: Color) {
        draw_texture_ex(
            &self.flag,
            dest.x,
            dest.y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                ..Default::default()
            },
        );
    }

    pub fn draw_item(&self, index: usize, dest: Rect, tint: Color) {
        draw_sheet(&self.items, ITEM_CELL, index, dest, false, tint);
    }
}

async fn texture(path: &str, name: &str) -> Texture2D {
    let texture = match macroquad::file::load_file(path).await {
        Ok(bytes) => match Image::from_file_with_format(&bytes, Some(ImageFormat::Png)) {
            Ok(image) => Texture2D::from_image(&image),
            Err(err) => {
                macroquad::logging::error!("failed to decode sprite '{name}' ({path}): {err}");
                fallback_texture()
            }
        },
        Err(err) => {
            macroquad::logging::error!("failed to load sprite '{name}' ({path}): {err}");
            fallback_texture()
        }
    };
    texture.set_filter(FilterMode::Nearest);
    texture
}

fn fallback_texture() -> Texture2D {
    // Visible magenta fallback instead of panicking in the browser.
    Texture2D::from_rgba8(1, 1, &[255, 0, 255, 255])
}

fn draw_sheet(texture: &Texture2D, cell: f32, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
    let source = Rect::new(frame as f32 * cell, 0.0, cell, cell);
    draw_texture_ex(
        texture,
        dest.x,
        dest.y,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(dest.w, dest.h)),
            source: Some(source),
            flip_x,
            ..Default::default()
        },
    );
}
