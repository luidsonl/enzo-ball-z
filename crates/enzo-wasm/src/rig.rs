//! Manifest-driven humanoid rig: per-state clip/frame resolution and layered composition.
//!
//! Rigs are loaded from `assets/entities/<id>/rig.json` (or `atlas.json` fallback)
//! with aligned layer textures. Missing manifests/layers degrade gracefully.
//! Embedded canonical humanoid frame/clip data ensures correct per-state frame
//! selection even when runtime manifest loading fails (e.g. stale build output).

use std::collections::HashMap;

use enzo_core::entities::VisualProfile;
use macroquad::prelude::*;
use serde::Deserialize;

const HUMANOID_CANONICAL_FRAMES: &[&str] = &[
    "idle_0",
    "idle_1",
    "walk_0",
    "walk_1",
    "walk_2",
    "walk_3",
    "run_0",
    "run_1",
    "run_2",
    "run_3",
    "crouch_0",
    "jump_rise_0",
    "jump_fall_0",
    "fly_0",
    "fly_1",
    "swim_0",
    "swim_1",
    "dash_0",
    "sixseven_a_0",
    "sixseven_b_0",
    "attack_1_0",
    "attack_2_0",
    "attack_3_0",
    "kick_0",
    "cast_0",
    "hurt_0",
    "dying_0",
    "dead_0",
];

fn canonical_clip(name: &str) -> Option<(&'static [&'static str], f32, bool)> {
    Some(match name {
        "idle" => (&["idle_0", "idle_1"][..], 2.5, true),
        "walk" => (&["walk_0", "walk_1", "walk_2", "walk_3"][..], 8.0, true),
        "run" => (&["run_0", "run_1", "run_2", "run_3"][..], 10.0, true),
        "crouch" => (&["crouch_0"][..], 1.0, true),
        "jump" | "jump_rise" => (&["jump_rise_0"][..], 1.0, true),
        "jump_fall" => (&["jump_fall_0"][..], 1.0, true),
        "fly" => (&["fly_0", "fly_1"][..], 6.0, true),
        "swim" => (&["swim_0", "swim_1"][..], 5.0, true),
        "dash" => (&["dash_0"][..], 1.0, true),
        "sixseven_a" => (&["sixseven_a_0"][..], 1.0, true),
        "sixseven_b" => (&["sixseven_b_0"][..], 1.0, true),
        "attack_1" => (&["attack_1_0"][..], 10.0, false),
        "attack_2" => (&["attack_2_0"][..], 10.0, false),
        "attack_3" => (&["attack_3_0"][..], 10.0, false),
        "kick" => (&["kick_0"][..], 10.0, false),
        "cast" => (&["cast_0"][..], 8.0, false),
        "hurt" => (&["hurt_0"][..], 1.0, true),
        "dying" => (&["dying_0"][..], 4.0, false),
        "dead" => (&["dead_0"][..], 1.0, true),
        _ => return None,
    })
}

fn canonical_fallbacks(name: &str) -> &'static [&'static str] {
    match name {
        "run" => &["walk", "idle"],
        "crouch" => &["idle"],
        "fly" => &["jump_rise", "jump_fall", "jump", "idle"],
        "swim" => &["fly", "walk", "idle"],
        "dash" => &["run", "walk", "idle"],
        "jump" => &["jump_rise", "jump_fall", "idle"],
        "attack_2" => &["attack_1"],
        "attack_3" => &["attack_2", "attack_1"],
        "kick" => &["attack_1"],
        "cast" => &["attack_1", "idle"],
        "dying" => &["dead", "hurt"],
        "sixseven_a" | "sixseven_b" => &["idle"],
        _ => &["idle"],
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl From<&RigRect> for Rect {
    fn from(r: &RigRect) -> Self {
        Rect::new(r.x, r.y, r.w, r.h)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigClip {
    pub frames: Vec<String>,
    #[serde(default = "default_fps")]
    pub fps: f32,
    #[serde(default = "default_loop")]
    pub r#loop: bool,
}

fn default_fps() -> f32 {
    1.0
}
fn default_loop() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigCell {
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigAnchor {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigManifest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub image: String,
    pub cell: RigCell,
    #[serde(default)]
    pub anchor: Option<RigAnchor>,
    #[serde(default)]
    pub frames: HashMap<String, RigRect>,
    #[serde(default)]
    pub clips: HashMap<String, RigClip>,
    #[serde(default)]
    pub fallbacks: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub layers: Vec<String>,
}

pub struct HumanoidRig {
    pub manifest: RigManifest,
    pub sheet: Texture2D, // full fallback
    pub base: Option<Texture2D>,
    pub face: Option<Texture2D>,
    pub hair: Option<Texture2D>,
    pub outfit_top: Option<Texture2D>,
    pub outfit_bottom: Option<Texture2D>,
    pub shoes: Option<Texture2D>,
}

impl HumanoidRig {
    pub async fn load(entity_id: &str, fallback_sheet: Texture2D) -> Self {
        let rig_path = format!("assets/entities/{entity_id}/rig.json");
        let atlas_path = format!("assets/entities/{entity_id}/atlas.json");
        let mut manifest = match load_manifest(&rig_path).await {
            Some(m) => m,
            None => load_manifest(&atlas_path).await.unwrap_or_else(|| {
                macroquad::logging::warn!(
                    "rig/atlas manifest missing for {entity_id}; using embedded canonical humanoid rig"
                );
                embedded_manifest(entity_id)
            }),
        };
        if manifest.frames.is_empty() || manifest.clips.is_empty() {
            let embedded = embedded_manifest(entity_id);
            if manifest.frames.is_empty() {
                manifest.frames = embedded.frames;
            }
            if manifest.clips.is_empty() {
                manifest.clips = embedded.clips;
            }
            if manifest.fallbacks.is_empty() {
                manifest.fallbacks = embedded.fallbacks;
            }
        }
        let base = opt_texture(&format!("assets/entities/{entity_id}/layers/base.png")).await;
        let face = opt_texture(&format!("assets/entities/{entity_id}/layers/face.png")).await;
        let hair = opt_texture(&format!("assets/entities/{entity_id}/layers/hair.png")).await;
        let outfit_top = opt_texture(&format!(
            "assets/entities/{entity_id}/layers/outfit_top.png"
        ))
        .await;
        let outfit_bottom = opt_texture(&format!(
            "assets/entities/{entity_id}/layers/outfit_bottom.png"
        ))
        .await;
        let shoes = opt_texture(&format!("assets/entities/{entity_id}/layers/shoes.png")).await;
        if base.is_none() {
            macroquad::logging::warn!(
                "humanoid customization layers missing for {entity_id}; composing from sheet frames"
            );
        }
        Self {
            manifest,
            sheet: fallback_sheet,
            base,
            face,
            hair,
            outfit_top,
            outfit_bottom,
            shoes,
        }
    }

    pub fn has_layers(&self) -> bool {
        self.base.is_some()
    }

    pub fn frame_rect(&self, clip_name: &str, time: f32) -> Option<Rect> {
        let frame_name = self.resolve_frame_name(clip_name, time)?;
        self.manifest
            .frames
            .get(&frame_name)
            .map(Rect::from)
            .or_else(|| {
                canonical_frame_rect(&frame_name, self.manifest.cell.w, self.manifest.cell.h)
            })
    }

    fn resolve_frame_name(&self, clip_name: &str, time: f32) -> Option<String> {
        if let Some(name) = self.resolve_clip_frame(clip_name, time) {
            return Some(name);
        }
        let from_manifest = self
            .manifest
            .fallbacks
            .get(clip_name)
            .map(|v| v.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        let fallback_list = from_manifest
            .as_deref()
            .unwrap_or_else(|| canonical_fallbacks(clip_name));
        for fb in fallback_list {
            if let Some(name) = self.resolve_clip_frame(fb, time) {
                return Some(name);
            }
        }
        for fb in ["idle", "walk"] {
            if let Some(name) = self.resolve_clip_frame(fb, time) {
                return Some(name);
            }
        }
        self.manifest.frames.keys().next().cloned()
    }

    fn resolve_clip_frame(&self, clip_name: &str, time: f32) -> Option<String> {
        if let Some(clip) = self.manifest.clips.get(clip_name) {
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
            return clip.frames.get(idx).cloned();
        }
        let (frames, fps, looping) = canonical_clip(clip_name)?;
        let idx = (time * fps.max(0.001)).floor() as usize;
        let idx = if looping {
            idx % frames.len()
        } else {
            idx.min(frames.len().saturating_sub(1))
        };
        frames.get(idx).map(|s| (*s).to_owned())
    }

    pub fn draw_composed(
        &self,
        clip_name: &str,
        time: f32,
        dest: Rect,
        flip_x: bool,
        tint: Color,
        profile: VisualProfile,
    ) {
        let Some(source) = self.frame_rect(clip_name, time) else {
            draw_sheet_tex(
                &self.sheet,
                Rect::new(0.0, 0.0, self.manifest.cell.w, self.manifest.cell.h),
                dest,
                flip_x,
                tint,
            );
            return;
        };
        if !self.has_layers() {
            draw_sheet_tex(&self.sheet, source, dest, flip_x, tint);
            return;
        }
        let skin_tint = skin_tint(profile.skin_tone, tint);
        let hair_tint = hair_tint(profile.hair_color, tint);
        if let Some(tex) = &self.base {
            draw_sheet_tex(tex, source, dest, flip_x, skin_tint);
        } else {
            draw_sheet_tex(&self.sheet, source, dest, flip_x, tint);
            return;
        }
        if let Some(tex) = &self.face {
            draw_sheet_tex(tex, source, dest, flip_x, tint);
        }
        if let Some(tex) = &self.outfit_bottom {
            draw_sheet_tex(tex, source, dest, flip_x, tint);
        }
        if let Some(tex) = &self.shoes {
            draw_sheet_tex(tex, source, dest, flip_x, tint);
        }
        if let Some(tex) = &self.outfit_top {
            draw_sheet_tex(tex, source, dest, flip_x, tint);
        }
        if let Some(tex) = &self.hair {
            draw_sheet_tex(tex, source, dest, flip_x, hair_tint);
        }
    }
}

fn embedded_manifest(entity_id: &str) -> RigManifest {
    let cell = 32.0;
    let mut frames = HashMap::new();
    for (i, name) in HUMANOID_CANONICAL_FRAMES.iter().enumerate() {
        frames.insert(
            (*name).to_owned(),
            RigRect {
                x: i as f32 * cell,
                y: 0.0,
                w: cell,
                h: cell,
            },
        );
    }
    let clips = [
        "idle",
        "walk",
        "run",
        "crouch",
        "jump",
        "jump_rise",
        "jump_fall",
        "fly",
        "swim",
        "dash",
        "sixseven_a",
        "sixseven_b",
        "attack_1",
        "attack_2",
        "attack_3",
        "kick",
        "cast",
        "hurt",
        "dying",
        "dead",
    ]
    .into_iter()
    .filter_map(|name| {
        let (f, fps, looping) = canonical_clip(name)?;
        Some((
            name.to_owned(),
            RigClip {
                frames: f.iter().map(|s| (*s).to_owned()).collect(),
                fps,
                r#loop: looping,
            },
        ))
    })
    .collect();
    let fallbacks = [
        ("run", vec!["walk", "idle"]),
        ("crouch", vec!["idle"]),
        ("fly", vec!["jump_rise", "jump_fall", "jump", "idle"]),
        ("swim", vec!["fly", "walk", "idle"]),
        ("dash", vec!["run", "walk", "idle"]),
        ("jump", vec!["jump_rise", "jump_fall", "idle"]),
        ("attack_2", vec!["attack_1"]),
        ("attack_3", vec!["attack_2", "attack_1"]),
        ("kick", vec!["attack_1"]),
        ("cast", vec!["attack_1", "idle"]),
        ("dying", vec!["dead", "hurt"]),
        ("sixseven_a", vec!["idle"]),
        ("sixseven_b", vec!["idle"]),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.into_iter().map(str::to_owned).collect()))
    .collect();
    RigManifest {
        id: entity_id.to_owned(),
        image: "sheet.png".to_owned(),
        cell: RigCell { w: cell, h: cell },
        anchor: Some(RigAnchor { x: 16.0, y: 30.0 }),
        frames,
        clips,
        fallbacks,
        layers: vec![],
    }
}

fn canonical_frame_rect(name: &str, w: f32, h: f32) -> Option<Rect> {
    let idx = HUMANOID_CANONICAL_FRAMES.iter().position(|f| *f == name)?;
    Some(Rect::new(idx as f32 * w, 0.0, w, h))
}

pub fn draw_sheet_tex(texture: &Texture2D, source: Rect, dest: Rect, flip_x: bool, tint: Color) {
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

async fn load_manifest(path: &str) -> Option<RigManifest> {
    let bytes = macroquad::file::load_file(path).await.ok()?;
    let text = String::from_utf8(bytes).ok()?;
    match serde_json::from_str::<RigManifest>(&text) {
        Ok(m) => Some(m),
        Err(e) => {
            macroquad::logging::warn!("failed to parse rig manifest {path}: {e}");
            None
        }
    }
}

pub async fn opt_texture(path: &str) -> Option<Texture2D> {
    let bytes = macroquad::file::load_file(path).await.ok()?;
    let image = Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).ok()?;
    let tex = Texture2D::from_image(&image);
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

fn skin_tint(id: u8, base: Color) -> Color {
    match id % 5 {
        0 => base,
        1 => Color::new(
            (base.r * 0.92).min(1.0),
            (base.g * 0.82).min(1.0),
            (base.b * 0.72).min(1.0),
            base.a,
        ),
        2 => Color::new(
            (base.r * 1.02).min(1.0),
            (base.g * 0.92).min(1.0),
            (base.b * 0.82).min(1.0),
            base.a,
        ),
        3 => Color::new(
            (base.r * 0.78).min(1.0),
            (base.g * 0.62).min(1.0),
            (base.b * 0.52).min(1.0),
            base.a,
        ),
        _ => Color::new(
            (base.r * 0.88).min(1.0),
            (base.g * 0.95).min(1.0),
            (base.b * 0.88).min(1.0),
            base.a,
        ),
    }
}

fn hair_tint(id: u8, base: Color) -> Color {
    match id % 6 {
        0 => base,
        1 => Color::new(0.20, 0.16, 0.14, base.a),
        2 => Color::new(0.85, 0.72, 0.35, base.a),
        3 => Color::new(0.55, 0.20, 0.18, base.a),
        4 => Color::new(0.75, 0.78, 0.85, base.a),
        _ => Color::new(0.35, 0.55, 0.85, base.a),
    }
}
