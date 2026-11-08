//! Lazy-loaded modular sprites/assets.
//!
//! Assets live under `assets/` in the new modular system. Textures and rigs are
//! requested on first render demand and loaded asynchronously by [`Sprites::pump`]
//! from the main async loop, with in-flight deduplication. Pending assets use
//! placeholders/procedural fallbacks.
//!
//! Legacy `assets/sprites/*` monolithic sheets are not used.

use std::cell::RefCell;

use enzo_core::entities::{ItemKind, VisualProfile};
use macroquad::prelude::*;

use crate::assets::{draw_lazy, pump_assets, AssetStore};
use crate::rig::HumanoidRig;

pub const TILE_CELL: f32 = 16.0;
pub const ACTOR_CELL: f32 = 32.0;
pub const ITEM_WORLD_SIZE: f32 = 16.0;

pub const TILES_SHEET_PATH: &str = "assets/blocks/tiles.png";
pub const SLIME_SHEET_PATH: &str = "assets/entities/slime/sheet.png";

/// Skill effect asset paths (new modular system).
pub mod skill_fx {
    pub const MOGGING_PROJECTILE: &str = "assets/skills/mogging/projectile.png";
    pub const MELEE_IMPACT: &str = "assets/skills/melee/impact.png";
    pub const NUTELLA_PROJECTILE: &str = "assets/skills/nutella_shot/projectile.png";
    pub const SHOUT_WAVE: &str = "assets/skills/shout/wave.png";
    pub const JUMP_DUST: &str = "assets/skills/jump/dust.png";
    pub const DASH_TRAIL: &str = "assets/skills/dash/trail.png";
}

/// Slime frame indices in `assets/entities/slime/sheet.png` (32x32 row).
pub mod slime {
    pub const WALK_A: usize = 0;
    pub const WALK_B: usize = 1;
    pub const STRETCH: usize = 2;
    pub const HURT: usize = 3;
}

/// Tile sheet frame indices in `assets/blocks/tiles.png`.
pub mod tile {
    pub const GROUND_TOP: usize = 0;
    pub const GROUND_FILL: usize = 1;
    pub const BLOCK: usize = 2;
    pub const PLATFORM: usize = 3;
    pub const CRATE: usize = 4;
    pub const CRATE_DAMAGED_1: usize = 5;
    pub const CRATE_DAMAGED_2: usize = 6;
    pub const FLAG_BOTTOM: usize = 7;
    pub const FLAG_TOP: usize = 8;
    pub const BREAKABLE_STONE: usize = 9;
    pub const BREAKABLE_STONE_DAMAGED_1: usize = 10;
    pub const BREAKABLE_STONE_DAMAGED_2: usize = 11;
}

#[derive(Clone)]
enum Lazy<T> {
    Empty,
    Queued,
    Ready(T),
}

struct LazyRig {
    id: &'static str,
    state: Lazy<HumanoidRig>,
}

impl LazyRig {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            state: Lazy::Empty,
        }
    }
}

pub struct Sprites {
    pub assets: AssetStore,
    enzo_rig: RefCell<LazyRig>,
    valentina_rig: RefCell<LazyRig>,
}

impl Sprites {
    pub fn new() -> Self {
        let assets = AssetStore::new();
        // Pre-request critical map assets; downloads are pumped asynchronously.
        assets.request(TILES_SHEET_PATH.to_owned());
        Self {
            assets,
            enzo_rig: RefCell::new(LazyRig::new("enzo")),
            valentina_rig: RefCell::new(LazyRig::new("valentina")),
        }
    }

    /// Progress lazy asset/rig loading. Call from the main async loop each frame.
    pub async fn pump(&self) {
        pump_assets(&self.assets, 2).await;
        self.pump_rig(&self.enzo_rig).await;
        self.pump_rig(&self.valentina_rig).await;
    }

    async fn pump_rig(&self, slot: &RefCell<LazyRig>) {
        let id = {
            let r = slot.borrow_mut();
            match r.state {
                Lazy::Queued => {
                    // leave as Queued during load marker? replace to avoid reentry
                    r.id
                }
                _ => return,
            }
        };
        // Mark as loading by taking state out via a sentinel - simplest: load and set Ready.
        let rig = HumanoidRig::load(id).await;
        let mut r = slot.borrow_mut();
        r.state = Lazy::Ready(rig);
    }

    fn ensure_rig(&self, slot: &RefCell<LazyRig>) {
        let mut r = slot.borrow_mut();
        if matches!(r.state, Lazy::Empty) {
            r.state = Lazy::Queued;
        }
    }

    pub fn tiles_texture(&self) -> Option<Texture2D> {
        self.assets.get_or_request(TILES_SHEET_PATH)
    }

    pub fn draw_enzo_rig(
        &self,
        clip: &str,
        time: f32,
        dest: Rect,
        flip_x: bool,
        tint: Color,
        profile: VisualProfile,
    ) {
        self.ensure_rig(&self.enzo_rig);
        let drawn = {
            let r = self.enzo_rig.borrow();
            if let Lazy::Ready(rig) = &r.state {
                rig.draw_composed(clip, time, dest, flip_x, tint, profile);
                true
            } else {
                false
            }
        };
        if !drawn {
            HumanoidRig::draw_placeholder(clip, time, dest, flip_x, tint);
        }
    }

    pub fn draw_valentina_rig(
        &self,
        clip: &str,
        time: f32,
        dest: Rect,
        flip_x: bool,
        tint: Color,
        profile: VisualProfile,
    ) {
        self.ensure_rig(&self.valentina_rig);
        let drawn = {
            let r = self.valentina_rig.borrow();
            if let Lazy::Ready(rig) = &r.state {
                rig.draw_composed(clip, time, dest, flip_x, tint, profile);
                true
            } else {
                false
            }
        };
        if !drawn {
            HumanoidRig::draw_placeholder(clip, time, dest, flip_x, tint);
        }
    }

    pub fn rig_frame_source(&self, character: &str, clip: &str, time: f32) -> Option<Rect> {
        let slot = if character == "valentina" {
            &self.valentina_rig
        } else {
            &self.enzo_rig
        };
        self.ensure_rig(slot);
        {
            let r = slot.borrow();
            if let Lazy::Ready(rig) = &r.state {
                return rig
                    .frame_rect(clip, time)
                    .or_else(|| embedded_frame_rect(clip, time));
            }
        }
        embedded_frame_rect(clip, time)
    }

    pub fn draw_slime(&self, frame: usize, dest: Rect, flip_x: bool, tint: Color) {
        let source = Rect::new(frame as f32 * ACTOR_CELL, 0.0, ACTOR_CELL, ACTOR_CELL);
        draw_lazy(
            &self.assets,
            SLIME_SHEET_PATH,
            Some(source),
            dest,
            flip_x,
            tint,
        );
    }

    /// Flag visuals are rendered as 2 map blocks (flag_bottom/flag_top). Keep a glow.
    pub fn draw_flag(&self, _dest: Rect, _tint: Color) {
        // no-op: blocks are in the tilemap; world draws pulse using snapshot rect.
        let _ = self;
    }

    pub fn draw_item_world(&self, kind: ItemKind, dest: Rect, tint: Color) {
        let path = format!("assets/items/{}/world.png", kind.id());
        if !draw_lazy(&self.assets, &path, None, dest, false, tint) {
            draw_rectangle(
                dest.x + 2.0,
                dest.y + 2.0,
                dest.w - 4.0,
                dest.h - 4.0,
                Color::from_rgba(246, 196, 69, 140),
            );
        }
    }

    pub fn draw_item_equipped_rig(
        &self,
        kind: ItemKind,
        source: Option<Rect>,
        dest: Rect,
        flip_x: bool,
        tint: Color,
    ) {
        let overlay = format!("assets/items/{}/layers/humanoid_overlay.png", kind.id());
        if let (Some(src), Some(tex)) = (source, self.assets.get_optional(&overlay)) {
            crate::rig::draw_sheet_tex(&tex, src, dest, flip_x, tint);
            return;
        }
        if source.is_some() && kind.data().slot().is_some() {
            let _ = self.assets.get_optional(&overlay);
        }
        let equipped = format!("assets/items/{}/equipped.png", kind.id());
        draw_lazy(&self.assets, &equipped, None, dest, flip_x, tint);
    }

    pub fn draw_skill_fx(&self, path: &str, dest: Rect, flip_x: bool, tint: Color) {
        if !draw_lazy(&self.assets, path, None, dest, flip_x, tint) {
            draw_circle(
                dest.x + dest.w * 0.5,
                dest.y + dest.h * 0.5,
                dest.w * 0.22,
                Color::new(tint.r, tint.g, tint.b, tint.a * 0.45),
            );
        }
    }
}

impl Default for Sprites {
    fn default() -> Self {
        Self::new()
    }
}

fn embedded_frame_rect(clip_name: &str, time: f32) -> Option<Rect> {
    let manifest = embedded_manifest();
    let clip = manifest
        .clips
        .get(clip_name)
        .or_else(|| manifest.clips.get("idle"))?;
    if clip.frames.is_empty() {
        return None;
    }
    let fps = if clip.fps <= 0.0 { 1.0 } else { clip.fps };
    let idx = (time * fps).floor() as usize;
    let idx = if clip.r#loop {
        idx % clip.frames.len()
    } else {
        idx.min(clip.frames.len().saturating_sub(1))
    };
    let name = clip.frames.get(idx)?;
    let r = manifest.frames.get(name)?;
    Some(Rect::new(r.x, r.y, r.w, r.h))
}

fn embedded_manifest() -> crate::rig::RigManifest {
    let json = serde_json::json!({
        "id": "humanoid",
        "image": "sheet.png",
        "cell": {"w": 32.0, "h": 32.0},
        "anchor": {"x": 16.0, "y": 30.0},
        "frames": {
            "idle_0":{"x":0.0,"y":0.0,"w":32.0,"h":32.0},"idle_1":{"x":32.0,"y":0.0,"w":32.0,"h":32.0},
            "walk_0":{"x":64.0,"y":0.0,"w":32.0,"h":32.0},"walk_1":{"x":96.0,"y":0.0,"w":32.0,"h":32.0},
            "walk_2":{"x":128.0,"y":0.0,"w":32.0,"h":32.0},"walk_3":{"x":160.0,"y":0.0,"w":32.0,"h":32.0},
            "run_0":{"x":192.0,"y":0.0,"w":32.0,"h":32.0},"run_1":{"x":224.0,"y":0.0,"w":32.0,"h":32.0},
            "run_2":{"x":256.0,"y":0.0,"w":32.0,"h":32.0},"run_3":{"x":288.0,"y":0.0,"w":32.0,"h":32.0},
            "crouch_0":{"x":320.0,"y":0.0,"w":32.0,"h":32.0},"jump_rise_0":{"x":352.0,"y":0.0,"w":32.0,"h":32.0},
            "jump_fall_0":{"x":384.0,"y":0.0,"w":32.0,"h":32.0},"fly_0":{"x":416.0,"y":0.0,"w":32.0,"h":32.0},
            "fly_1":{"x":448.0,"y":0.0,"w":32.0,"h":32.0},"swim_0":{"x":480.0,"y":0.0,"w":32.0,"h":32.0},
            "swim_1":{"x":512.0,"y":0.0,"w":32.0,"h":32.0},"dash_0":{"x":544.0,"y":0.0,"w":32.0,"h":32.0},
            "sixseven_a_0":{"x":576.0,"y":0.0,"w":32.0,"h":32.0},"sixseven_b_0":{"x":608.0,"y":0.0,"w":32.0,"h":32.0},
            "attack_1_0":{"x":640.0,"y":0.0,"w":32.0,"h":32.0},"attack_2_0":{"x":672.0,"y":0.0,"w":32.0,"h":32.0},
            "attack_3_0":{"x":704.0,"y":0.0,"w":32.0,"h":32.0},"kick_0":{"x":736.0,"y":0.0,"w":32.0,"h":32.0},
            "cast_0":{"x":768.0,"y":0.0,"w":32.0,"h":32.0},"hurt_0":{"x":800.0,"y":0.0,"w":32.0,"h":32.0},
            "dying_0":{"x":832.0,"y":0.0,"w":32.0,"h":32.0},"dead_0":{"x":864.0,"y":0.0,"w":32.0,"h":32.0}
        },
        "clips": {
            "idle":{"frames":["idle_0","idle_1"],"fps":2.5,"loop":true},
            "walk":{"frames":["walk_0","walk_1","walk_2","walk_3"],"fps":8.0,"loop":true},
            "run":{"frames":["run_0","run_1","run_2","run_3"],"fps":10.0,"loop":true},
            "crouch":{"frames":["crouch_0"],"fps":1.0,"loop":true},
            "jump":{"frames":["jump_rise_0"],"fps":1.0,"loop":true},
            "jump_rise":{"frames":["jump_rise_0"],"fps":1.0,"loop":true},
            "jump_fall":{"frames":["jump_fall_0"],"fps":1.0,"loop":true},
            "fly":{"frames":["fly_0","fly_1"],"fps":6.0,"loop":true},
            "swim":{"frames":["swim_0","swim_1"],"fps":5.0,"loop":true},
            "dash":{"frames":["dash_0"],"fps":1.0,"loop":true},
            "sixseven_a":{"frames":["sixseven_a_0"],"fps":1.0,"loop":true},
            "sixseven_b":{"frames":["sixseven_b_0"],"fps":1.0,"loop":true},
            "attack_1":{"frames":["attack_1_0"],"fps":10.0,"loop":false},
            "attack_2":{"frames":["attack_2_0"],"fps":10.0,"loop":false},
            "attack_3":{"frames":["attack_3_0"],"fps":10.0,"loop":false},
            "kick":{"frames":["kick_0"],"fps":10.0,"loop":false},
            "cast":{"frames":["cast_0"],"fps":8.0,"loop":false},
            "hurt":{"frames":["hurt_0"],"fps":1.0,"loop":true},
            "dying":{"frames":["dying_0"],"fps":4.0,"loop":false},
            "dead":{"frames":["dead_0"],"fps":1.0,"loop":true}
        },
        "fallbacks": {"run":["walk","idle"],"crouch":["idle"],"fly":["jump_rise","jump_fall","jump","idle"],"swim":["fly","walk","idle"],"dash":["run","walk","idle"],"jump":["jump_rise","jump_fall","idle"],"attack_2":["attack_1"],"attack_3":["attack_2","attack_1"],"kick":["attack_1"],"cast":["attack_1","idle"],"dying":["dead","hurt"],"sixseven_a":["idle"],"sixseven_b":["idle"]},
        "layers": []
    });
    serde_json::from_value(json).expect("embedded canonical rig manifest")
}
