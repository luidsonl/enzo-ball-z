//! Sprites originais embutidos no binário.
//!
//! Os PNGs são gerados por `scripts/gen-sprites.mjs` e entram no wasm via
//! `include_bytes!`. Assim o cliente não depende de carregamento assíncrono de
//! assets nem de caminhos relativos no navegador.
//!
//! Cada folha usa uma célula fixa: tiles e itens têm 16×16; atores e efeitos
//! têm 32×32. O render escolhe o frame pelo estado do snapshot.

use macroquad::prelude::*;

const TILES_PNG: &[u8] = include_bytes!("../../../assets/sprites/tiles.png");
const ENZO_PNG: &[u8] = include_bytes!("../../../assets/sprites/enzo.png");
const SLIME_PNG: &[u8] = include_bytes!("../../../assets/sprites/slime.png");
const FX_PNG: &[u8] = include_bytes!("../../../assets/sprites/fx.png");
const FLAG_PNG: &[u8] = include_bytes!("../../../assets/sprites/flag.png");
const ITEMS_PNG: &[u8] = include_bytes!("../../../assets/sprites/items.png");

pub const TILE_CELL: f32 = 16.0;
pub const ACTOR_CELL: f32 = 32.0;
pub const ITEM_CELL: f32 = 16.0;

/// Índices das folhas de tiles.
pub mod tile {
    pub const GROUND_TOP: usize = 0;
    pub const GROUND_FILL: usize = 1;
    pub const BLOCK: usize = 2;
    pub const PLATFORM: usize = 3;
}

/// Índices da folha do Enzo.
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

/// Índices da folha da capanga.
pub mod slime {
    pub const WALK_A: usize = 0;
    pub const WALK_B: usize = 1;
    pub const STRETCH: usize = 2;
    pub const HURT: usize = 3;
}

/// Índices da folha de efeitos.
pub mod fx {
    pub const MOGAR: usize = 0;
    pub const SLASH_A: usize = 1;
    pub const SLASH_B: usize = 2;
    pub const SLASH_C: usize = 3;
    pub const SPARK_YELLOW: usize = 4;
    pub const SPARK_CYAN: usize = 5;
    pub const SPARK_RED: usize = 6;
    pub const DUST: usize = 7;
}

/// Índices da folha de itens.
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
    slime: Texture2D,
    fx: Texture2D,
    flag: Texture2D,
    items: Texture2D,
}

impl Sprites {
    pub fn load() -> Self {
        Self {
            tiles: texture(TILES_PNG, "tiles"),
            enzo: texture(ENZO_PNG, "enzo"),
            slime: texture(SLIME_PNG, "slime"),
            fx: texture(FX_PNG, "fx"),
            flag: texture(FLAG_PNG, "flag"),
            items: texture(ITEMS_PNG, "items"),
        }
    }

    pub fn draw_tile(&self, index: usize, dest: Rect, tint: Color) {
        draw_sheet(&self.tiles, TILE_CELL, index, dest, false, tint);
    }

    /// Textura usada pelo cache de chunks de tiles.
    pub fn tiles_texture(&self) -> &Texture2D {
        &self.tiles
    }

    pub fn draw_enzo(&self, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
        draw_sheet(&self.enzo, ACTOR_CELL, frame, dest, flip_x, tint);
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

fn texture(bytes: &[u8], name: &str) -> Texture2D {
    let texture = match Image::from_file_with_format(bytes, Some(ImageFormat::Png)) {
        Ok(image) => Texture2D::from_image(&image),
        Err(err) => {
            macroquad::logging::error!("falha ao carregar sprite '{name}': {err}");
            // Fallback visível (magenta) em vez de panic no navegador.
            Texture2D::from_rgba8(1, 1, &[255, 0, 255, 255])
        }
    };
    texture.set_filter(FilterMode::Nearest);
    texture
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
