//! Lazy-loaded modular sprites/assets.
//!
//! Assets/textures are requested on demand and loaded via non-blocking polling
//! from the main loop, so rendering continues with placeholders while downloads
//! complete (no black flashes from blocked frames).
//!
//! Legacy `assets/sprites/*` monolithic sheets are not used.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use enzo_core::entities::{ItemKind, VisualProfile};
use macroquad::prelude::*;

use crate::assets::{draw_lazy, pump_assets, AssetStore};
use crate::rig::{HumanoidRig, ResolvedFrame};

pub const TILE_CELL: f32 = 16.0;
pub const HUMANOID_ACTOR_CELL: f32 = 64.0;
pub const ACTOR_CELL: f32 = 32.0;
pub const ITEM_WORLD_SIZE: f32 = 16.0;

pub const TILES_SHEET_PATH: &str = "assets/blocks/tiles.png";
pub const SLIME_SHEET_PATH: &str = "assets/entities/slime/sheet.png";

pub mod skill_fx {
    pub const MOGGING_PROJECTILE: &str = "assets/skills/mogging/projectile.png";
    pub const MELEE_IMPACT: &str = "assets/skills/melee/impact.png";
    pub const NUTELLA_PROJECTILE: &str = "assets/skills/nutella_shot/projectile.png";
    pub const SHOUT_WAVE: &str = "assets/skills/shout/wave.png";
    pub const JUMP_DUST: &str = "assets/skills/jump/dust.png";
    pub const DASH_TRAIL: &str = "assets/skills/dash/trail.png";
}

pub mod slime {
    pub const WALK_A: usize = 0;
    pub const WALK_B: usize = 1;
    pub const STRETCH: usize = 2;
    pub const HURT: usize = 3;
}

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

type BoxFut<T> = Pin<Box<dyn Future<Output = T>>>;

fn noop_waker() -> Waker {
    fn clone(_: *const ()) -> RawWaker {
        RawWaker::new(std::ptr::null(), &VTABLE)
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);
    unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) }
}

fn poll_once<T>(fut: &mut BoxFut<T>) -> Poll<T> {
    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    fut.as_mut().poll(&mut cx)
}

enum RigState {
    Empty,
    Loading(BoxFut<HumanoidRig>),
    Ready(Box<HumanoidRig>),
}

struct LazyRig {
    id: &'static str,
    state: RigState,
}

impl LazyRig {
    fn new(id: &'static str) -> Self {
        Self {
            id,
            state: RigState::Empty,
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
        assets.request(TILES_SHEET_PATH.to_owned());
        Self {
            assets,
            enzo_rig: RefCell::new(LazyRig::new("enzo")),
            valentina_rig: RefCell::new(LazyRig::new("valentina")),
        }
    }

    /// Progress lazy asset/rig loading without blocking. Renders use placeholders meanwhile.
    pub fn pump(&self) {
        pump_assets(&self.assets, 2);
        self.pump_rig(&self.enzo_rig);
        self.pump_rig(&self.valentina_rig);
    }

    fn pump_rig(&self, slot: &RefCell<LazyRig>) {
        let mut r = slot.borrow_mut();
        let (id, should_start) = match &r.state {
            RigState::Empty => (r.id, true),
            RigState::Loading(_) => (r.id, false),
            RigState::Ready(_) => return,
        };
        if should_start {
            let fut = Box::pin(HumanoidRig::load(id));
            r.state = RigState::Loading(fut);
            return;
        }
        if let RigState::Loading(fut) = &mut r.state {
            if let Poll::Ready(rig) = poll_once(fut) {
                r.state = RigState::Ready(Box::new(rig));
            }
        }
    }

    fn ensure_rig(&self, slot: &RefCell<LazyRig>) {
        self.pump_rig(slot);
    }

    pub fn tiles_texture(&self) -> Option<Texture2D> {
        self.assets.get_or_request(TILES_SHEET_PATH)
    }

    fn profile_hair_texture(&self, character: &str, profile: VisualProfile) -> Option<Texture2D> {
        let path = format!(
            "assets/entities/{character}/layers/hair_style_{}.png",
            profile.hair_style % 8
        );
        self.assets.get_or_request(&path)
    }

    fn profile_face_texture(&self, character: &str, profile: VisualProfile) -> Option<Texture2D> {
        let path = format!(
            "assets/entities/{character}/layers/face_{}.png",
            profile.face % 8
        );
        self.assets.get_or_request(&path)
    }

    /// Resolves a humanoid frame and draws it anchored to entity feet (bottom-center).
    /// Returns the resolved frame (for aligned overlays) if drawn from a ready rig.
    pub fn draw_enzo_rig(
        &self,
        clip: &str,
        time: f32,
        foot: Vec2,
        flip_x: bool,
        tint: Color,
        profile: VisualProfile,
    ) -> Option<ResolvedFrame> {
        self.ensure_rig(&self.enzo_rig);
        let hair = self.profile_hair_texture("enzo", profile);
        let face = self.profile_face_texture("enzo", profile);
        let r = self.enzo_rig.borrow();
        if let RigState::Ready(rig) = &r.state {
            if let Some(frame) = rig.resolve_frame(clip, time) {
                let dest = frame.dest_for_foot(foot.x, foot.y);
                rig.draw_composed(
                    clip,
                    time,
                    dest,
                    flip_x,
                    tint,
                    profile,
                    hair.as_ref(),
                    face.as_ref(),
                );
                return Some(frame);
            }
        }
        // Placeholder/fallback (legacy 64x64 anchored at feet).
        let dest = Rect::new(foot.x - 32.0, foot.y - 62.0, 64.0, 64.0);
        HumanoidRig::draw_placeholder(clip, time, dest, flip_x, tint);
        None
    }

    pub fn draw_valentina_rig(
        &self,
        clip: &str,
        time: f32,
        foot: Vec2,
        flip_x: bool,
        tint: Color,
        profile: VisualProfile,
    ) -> Option<ResolvedFrame> {
        self.ensure_rig(&self.valentina_rig);
        let hair = self.profile_hair_texture("valentina", profile);
        let face = self.profile_face_texture("valentina", profile);
        let r = self.valentina_rig.borrow();
        if let RigState::Ready(rig) = &r.state {
            if let Some(frame) = rig.resolve_frame(clip, time) {
                let dest = frame.dest_for_foot(foot.x, foot.y);
                rig.draw_composed(
                    clip,
                    time,
                    dest,
                    flip_x,
                    tint,
                    profile,
                    hair.as_ref(),
                    face.as_ref(),
                );
                return Some(frame);
            }
        }
        let dest = Rect::new(foot.x - 32.0, foot.y - 62.0, 64.0, 64.0);
        HumanoidRig::draw_placeholder(clip, time, dest, flip_x, tint);
        None
    }

    /// Resolves current humanoid frame (source/layout) without drawing.
    pub fn rig_frame(&self, character: &str, clip: &str, time: f32) -> Option<ResolvedFrame> {
        let slot = if character == "valentina" {
            &self.valentina_rig
        } else {
            &self.enzo_rig
        };
        self.ensure_rig(slot);
        let r = slot.borrow();
        if let RigState::Ready(rig) = &r.state {
            return rig.resolve_frame(clip, time);
        }
        None
    }

    /// Compatibility: source rect only.
    pub fn rig_frame_source(&self, character: &str, clip: &str, time: f32) -> Option<Rect> {
        self.rig_frame(character, clip, time).map(|f| f.source)
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

    pub fn draw_flag(&self, _dest: Rect, _tint: Color) {
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
        let Some(src) = source else {
            return;
        };
        // Legacy humanoid overlay sheets are strip-aligned (896x32, 32x32 cells) and not
        // compatible with matrix rigs/state-specific frame layouts yet. Only the legacy
        // 32x32 cell path could sample them safely; skip everything else to avoid
        // stretching several strip frames across wide low-state frames (crouch/crawl).
        if src.w != 32.0 || src.h != 32.0 {
            return;
        }
        let overlay = format!("assets/items/{}/layers/humanoid_overlay.png", kind.id());
        if let Some(tex) = self.assets.get_optional(&overlay) {
            crate::rig::draw_sheet_tex(&tex, src, dest, flip_x, tint);
            return;
        }
        if kind.data().slot().is_some() {
            let _ = self.assets.get_or_request(&overlay);
        }
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

// (embedded strip fallback removed: matrix rigs resolve from manifest; loading shows placeholder)
