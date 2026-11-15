//! Manifest-driven humanoid rig: per-state clip/frame resolution and layered composition.
//!
//! Supports:
//! - Matrix spritesheets (`layout: "state_matrix"`): rows = states, cols = frames, per-state
//!   frame dimensions/anchors and per-state frame duration.
//! - Legacy strip manifests (flat frames + uniform cell).
//!
//! Rigs are loaded from `assets/entities/<id>/rig.json` (or `atlas.json` fallback)
//! with aligned layer textures. Missing manifests/layers degrade gracefully.

use std::collections::HashMap;

use enzo_core::entities::VisualProfile;
use macroquad::prelude::*;
use serde::Deserialize;

#[derive(Clone, Copy, Debug)]
pub struct ResolvedFrame {
    pub source: Rect,
    pub frame_w: f32,
    pub frame_h: f32,
    pub anchor_x: f32,
    pub anchor_y: f32,
}

impl ResolvedFrame {
    pub fn dest_for_foot(self, foot_x: f32, foot_y: f32) -> Rect {
        Rect::new(
            foot_x - self.anchor_x,
            foot_y - self.anchor_y,
            self.frame_w,
            self.frame_h,
        )
    }
}

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
    "crawl_0",
    "crawl_1",
    "crawl_2",
    "crawl_3",
    "jump_rise_0",
    "jump_fall_0",
    "fly_0",
    "fly_1",
    "swim_0",
    "swim_1",
    "climb_0",
    "climb_1",
    "dash_0",
    "sixseven_a_0",
    "sixseven_b_0",
    "attack_1_0",
    "attack_2_0",
    "attack_3_0",
    "kick_0",
    "cast_0",
    "shout_0",
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
        "crawl" => (&["crawl_0", "crawl_1", "crawl_2", "crawl_3"][..], 7.0, true),
        "jump" | "jump_rise" => (&["jump_rise_0"][..], 1.0, true),
        "jump_fall" => (&["jump_fall_0"][..], 1.0, true),
        "fly" => (&["fly_0", "fly_1"][..], 6.0, true),
        "swim" => (&["swim_0", "swim_1"][..], 5.0, true),
        "climb" => (&["climb_0", "climb_1"][..], 5.0, true),
        "dash" => (&["dash_0"][..], 1.0, true),
        "sixseven_a" => (&["sixseven_a_0"][..], 1.0, true),
        "sixseven_b" => (&["sixseven_b_0"][..], 1.0, true),
        "attack_1" => (&["attack_1_0"][..], 10.0, false),
        "attack_2" => (&["attack_2_0"][..], 10.0, false),
        "attack_3" => (&["attack_3_0"][..], 10.0, false),
        "kick" => (&["kick_0"][..], 10.0, false),
        "cast" => (&["cast_0"][..], 8.0, false),
        "shout" => (&["shout_0"][..], 8.0, false),
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
        "crawl" => &["crouch", "walk", "idle"],
        "fly" => &["jump_rise", "jump_fall", "jump", "idle"],
        "swim" => &["fly", "walk", "idle"],
        "climb" => &["jump_rise", "idle"],
        "dash" => &["run", "walk", "idle"],
        "jump" => &["jump_rise", "jump_fall", "idle"],
        "attack_2" => &["attack_1"],
        "attack_3" => &["attack_2", "attack_1"],
        "kick" => &["attack_1"],
        "cast" => &["attack_1", "idle"],
        "shout" => &["cast", "attack_1", "idle"],
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

#[derive(Clone, Debug, Deserialize, Default)]
pub struct RigAnchor {
    #[serde(default)]
    pub x: f32,
    #[serde(default)]
    pub y: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigClip {
    pub frames: Vec<String>,
    #[serde(default = "default_fps")]
    pub fps: f32,
    #[serde(default)]
    pub frame_duration: Option<f32>,
    #[serde(default = "default_loop")]
    pub r#loop: bool,
}

impl RigClip {
    fn fps_resolved(&self) -> f32 {
        if let Some(d) = self.frame_duration {
            if d > 0.0 {
                return 1.0 / d;
            }
        }
        if self.fps > 0.0 {
            self.fps
        } else {
            1.0
        }
    }
}

fn default_fps() -> f32 {
    1.0
}
fn default_loop() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, Default)]
pub struct RigCell {
    #[serde(default = "default_cell")]
    pub w: f32,
    #[serde(default = "default_cell")]
    pub h: f32,
}
fn default_cell() -> f32 {
    64.0
}

// --- Matrix (state rows x frame columns) contract ---

#[derive(Clone, Debug, Deserialize)]
pub struct MatrixFrame {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    #[serde(default)]
    pub anchor: Option<RigAnchor>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StateRow {
    #[serde(rename = "frameW", default)]
    pub frame_w: f32,
    #[serde(rename = "frameH", default)]
    pub frame_h: f32,
    #[serde(default)]
    pub anchor: Option<RigAnchor>,
    #[serde(default)]
    pub frames: Vec<MatrixFrame>,
}

#[derive(Clone, Debug)]
struct FrameEntry {
    source: Rect,
    frame_w: f32,
    frame_h: f32,
    anchor: RigAnchor,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RigManifest {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub layout: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub cell: Option<RigCell>,
    #[serde(default)]
    pub anchor: Option<RigAnchor>,
    /// Legacy flat frame map.
    #[serde(default)]
    pub frames: HashMap<String, RigRect>,
    /// Matrix layout: state rows with frame rects.
    #[serde(default)]
    pub states: HashMap<String, StateRow>,
    #[serde(default)]
    pub clips: HashMap<String, RigClip>,
    #[serde(default)]
    pub fallbacks: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub layers: Vec<String>,
}

impl RigManifest {
    fn default_anchor(&self) -> RigAnchor {
        self.anchor
            .clone()
            .unwrap_or(RigAnchor { x: 32.0, y: 62.0 })
    }
    fn default_cell(&self) -> (f32, f32) {
        if let Some(c) = &self.cell {
            (c.w, c.h)
        } else {
            (64.0, 64.0)
        }
    }
}

pub struct HumanoidRig {
    pub manifest: RigManifest,
    /// Flat frame lookup (covers matrix + legacy manifests).
    frame_map: HashMap<String, FrameEntry>,
    pub sheet: Texture2D, // full fallback
    pub base: Option<Texture2D>,
    pub face: Option<Texture2D>,
    pub hair: Option<Texture2D>,
    pub outfit_top: Option<Texture2D>,
    pub outfit_bottom: Option<Texture2D>,
    pub shoes: Option<Texture2D>,
}

impl HumanoidRig {
    pub async fn load(entity_id: &str) -> Self {
        let sheet_path = format!("assets/entities/{entity_id}/sheet.png");
        let sheet = opt_texture(&sheet_path)
            .await
            .unwrap_or_else(|| Texture2D::from_rgba8(1, 1, &[255, 0, 255, 255]));
        sheet.set_filter(FilterMode::Nearest);
        let rig_path = format!("assets/entities/{entity_id}/rig.json");
        let atlas_path = format!("assets/entities/{entity_id}/atlas.json");
        let mut manifest = match load_manifest(&rig_path).await {
            Some(m) => m,
            None => load_manifest(&atlas_path)
                .await
                .unwrap_or_else(|| embedded_manifest(entity_id)),
        };
        if manifest.frames.is_empty() && manifest.states.is_empty() {
            let embedded = embedded_manifest(entity_id);
            manifest.frames = embedded.frames;
            if manifest.clips.is_empty() {
                manifest.clips = embedded.clips;
            }
            if manifest.fallbacks.is_empty() {
                manifest.fallbacks = embedded.fallbacks;
            }
        }
        if manifest.clips.is_empty() {
            manifest.clips = embedded_manifest(entity_id).clips;
        }
        if manifest.fallbacks.is_empty() {
            manifest.fallbacks = embedded_manifest(entity_id).fallbacks;
        }
        let frame_map = build_frame_map(&manifest);
        Self {
            frame_map,
            manifest,
            sheet,
            base: opt_texture(&format!("assets/entities/{entity_id}/layers/base.png")).await,
            face: opt_texture(&format!("assets/entities/{entity_id}/layers/face.png")).await,
            hair: opt_texture(&format!("assets/entities/{entity_id}/layers/hair.png")).await,
            outfit_top: opt_texture(&format!(
                "assets/entities/{entity_id}/layers/outfit_top.png"
            ))
            .await,
            outfit_bottom: opt_texture(&format!(
                "assets/entities/{entity_id}/layers/outfit_bottom.png"
            ))
            .await,
            shoes: opt_texture(&format!("assets/entities/{entity_id}/layers/shoes.png")).await,
        }
    }

    /// Draws a loading placeholder (no texture dependency).
    pub fn draw_placeholder(_clip_name: &str, _time: f32, dest: Rect, _flip_x: bool, tint: Color) {
        let a = Color::new(tint.r, tint.g, tint.b, (tint.a * 0.35).clamp(0.0, 1.0));
        draw_rectangle(
            dest.x + dest.w * 0.22,
            dest.y + dest.h * 0.08,
            dest.w * 0.56,
            dest.h * 0.84,
            a,
        );
        draw_rectangle_lines(
            dest.x,
            dest.y,
            dest.w,
            dest.h,
            1.0,
            Color::new(1.0, 1.0, 1.0, 0.18),
        );
    }

    pub fn has_layers(&self) -> bool {
        self.base.is_some()
    }

    pub fn resolve_frame(&self, clip_name: &str, time: f32) -> Option<ResolvedFrame> {
        let frame_name = self.resolve_frame_name(clip_name, time)?;
        if let Some(e) = self.frame_map.get(&frame_name) {
            return Some(ResolvedFrame {
                source: e.source,
                frame_w: e.frame_w,
                frame_h: e.frame_h,
                anchor_x: e.anchor.x,
                anchor_y: e.anchor.y,
            });
        }
        // Legacy fallback by canonical strip index + manifest cell.
        let (cw, ch) = self.manifest.default_cell();
        let rect = canonical_frame_rect(&frame_name, cw, ch)?;
        let a = self.manifest.default_anchor();
        Some(ResolvedFrame {
            source: rect,
            frame_w: cw,
            frame_h: ch,
            anchor_x: a.x,
            anchor_y: a.y,
        })
    }

    pub fn frame_rect(&self, clip_name: &str, time: f32) -> Option<Rect> {
        self.resolve_frame(clip_name, time).map(|f| f.source)
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
        self.frame_map.keys().next().cloned()
    }

    fn resolve_clip_frame(&self, clip_name: &str, time: f32) -> Option<String> {
        if let Some(clip) = self.manifest.clips.get(clip_name) {
            if clip.frames.is_empty() {
                return None;
            }
            let fps = clip.fps_resolved().max(0.001);
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

    #[allow(clippy::too_many_arguments)]
    pub fn draw_composed(
        &self,
        clip_name: &str,
        time: f32,
        dest: Rect,
        flip_x: bool,
        tint: Color,
        profile: VisualProfile,
        hair_override: Option<&Texture2D>,
        face_override: Option<&Texture2D>,
    ) {
        let (source, fallback_dest) = match self.resolve_frame(clip_name, time) {
            Some(f) => (f.source, Rect::new(dest.x, dest.y, f.frame_w, f.frame_h)),
            None => {
                let (cw, ch) = self.manifest.default_cell();
                (
                    Rect::new(0.0, 0.0, cw, ch),
                    Rect::new(dest.x, dest.y, cw, ch),
                )
            }
        };
        // Caller supplies correctly positioned/sized dest (matrix-aware).
        let draw_dest = dest;
        if !self.has_layers() {
            draw_sheet_tex(&self.sheet, source, draw_dest, flip_x, tint);
            return;
        }
        let skin = skin_tint(profile.skin_tone, tint);
        let hair = hair_tint(profile.hair_color, tint);
        let outfit_main = clothing_tint(profile.outfit_top, tint);
        let outfit_bottom = clothing_tint(profile.outfit_bottom, tint);
        let detail_color = clothing_tint(profile.shoes, tint);
        if let Some(tex) = &self.base {
            draw_sheet_tex(tex, source, draw_dest, flip_x, skin);
        } else {
            draw_sheet_tex(&self.sheet, source, draw_dest, flip_x, tint);
            return;
        }
        let face_tex = face_override.or(self.face.as_ref());
        if let Some(tex) = face_tex {
            draw_sheet_tex(tex, source, draw_dest, flip_x, tint);
        }
        if let Some(tex) = &self.outfit_bottom {
            draw_sheet_tex(tex, source, draw_dest, flip_x, outfit_bottom);
        }
        if let Some(tex) = &self.shoes {
            draw_sheet_tex(tex, source, draw_dest, flip_x, detail_color);
        }
        if let Some(tex) = &self.outfit_top {
            draw_sheet_tex(tex, source, draw_dest, flip_x, outfit_main);
        }
        let hair_tex = hair_override.or(self.hair.as_ref());
        if let Some(tex) = hair_tex {
            draw_sheet_tex(tex, source, draw_dest, flip_x, hair);
        }
        let _ = fallback_dest;
    }
}

fn build_frame_map(manifest: &RigManifest) -> HashMap<String, FrameEntry> {
    let mut map = HashMap::new();
    let default_anchor = manifest.default_anchor();
    // Matrix states -> flat frame entries.
    for row in manifest.states.values() {
        let row_anchor = row.anchor.clone().unwrap_or_else(|| default_anchor.clone());
        for f in &row.frames {
            let a = f.anchor.clone().unwrap_or_else(|| row_anchor.clone());
            map.insert(
                f.name.clone(),
                FrameEntry {
                    source: Rect::new(f.x, f.y, f.w, f.h),
                    frame_w: f.w,
                    frame_h: f.h,
                    anchor: a,
                },
            );
        }
    }
    if !map.is_empty() {
        return map;
    }
    // Legacy flat frames + uniform cell.
    let (cw, ch) = manifest.default_cell();
    for (name, r) in &manifest.frames {
        map.insert(
            name.clone(),
            FrameEntry {
                source: Rect::from(r),
                frame_w: r.w.max(cw),
                frame_h: r.h.max(ch),
                anchor: default_anchor.clone(),
            },
        );
    }
    map
}

fn embedded_manifest(entity_id: &str) -> RigManifest {
    let cell = 64.0;
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
        "crawl",
        "jump",
        "jump_rise",
        "jump_fall",
        "fly",
        "swim",
        "climb",
        "dash",
        "sixseven_a",
        "sixseven_b",
        "attack_1",
        "attack_2",
        "attack_3",
        "kick",
        "cast",
        "shout",
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
                frame_duration: None,
                r#loop: looping,
            },
        ))
    })
    .collect();
    let fallbacks = [
        ("run", vec!["walk", "idle"]),
        ("crouch", vec!["idle"]),
        ("crawl", vec!["crouch", "walk", "idle"]),
        ("fly", vec!["jump_rise", "jump_fall", "jump", "idle"]),
        ("swim", vec!["fly", "walk", "idle"]),
        ("climb", vec!["jump_rise", "idle"]),
        ("dash", vec!["run", "walk", "idle"]),
        ("jump", vec!["jump_rise", "jump_fall", "idle"]),
        ("attack_2", vec!["attack_1"]),
        ("attack_3", vec!["attack_2", "attack_1"]),
        ("kick", vec!["attack_1"]),
        ("cast", vec!["attack_1", "idle"]),
        ("shout", vec!["cast", "attack_1", "idle"]),
        ("dying", vec!["dead", "hurt"]),
        ("sixseven_a", vec!["idle"]),
        ("sixseven_b", vec!["idle"]),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.into_iter().map(str::to_owned).collect()))
    .collect();
    RigManifest {
        id: entity_id.to_owned(),
        layout: "legacy_strip".to_owned(),
        image: "sheet.png".to_owned(),
        cell: Some(RigCell { w: cell, h: cell }),
        anchor: Some(RigAnchor { x: 32.0, y: 62.0 }),
        frames,
        states: HashMap::new(),
        clips,
        fallbacks,
        layers: vec![
            "base".into(),
            "face".into(),
            "hair".into(),
            "outfit_bottom".into(),
            "shoes".into(),
            "outfit_top".into(),
        ],
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

fn palette_tint(base: Color, r: f32, g: f32, b: f32) -> Color {
    Color::new(
        (base.r * r).clamp(0.0, 1.0),
        (base.g * g).clamp(0.0, 1.0),
        (base.b * b).clamp(0.0, 1.0),
        base.a,
    )
}

fn skin_tint(id: u8, base: Color) -> Color {
    match id % 8 {
        0 => base,
        1 => palette_tint(base, 1.08, 0.96, 0.88),
        2 => palette_tint(base, 0.95, 0.84, 0.74),
        3 => palette_tint(base, 0.86, 0.72, 0.60),
        4 => palette_tint(base, 0.74, 0.58, 0.46),
        5 => palette_tint(base, 0.62, 0.46, 0.36),
        6 => palette_tint(base, 0.90, 0.92, 0.84),
        _ => palette_tint(base, 1.00, 0.86, 0.82),
    }
}

fn hair_tint(id: u8, base: Color) -> Color {
    let a = base.a;
    match id % 8 {
        0 => base,
        1 => Color::new(0.12, 0.10, 0.10, a),
        2 => Color::new(0.36, 0.24, 0.16, a),
        3 => Color::new(0.82, 0.68, 0.30, a),
        4 => Color::new(0.62, 0.22, 0.18, a),
        5 => Color::new(0.78, 0.80, 0.86, a),
        6 => Color::new(0.30, 0.52, 0.88, a),
        _ => Color::new(0.72, 0.38, 0.82, a),
    }
}

fn clothing_tint(id: u8, base: Color) -> Color {
    let a = base.a;
    match id % 8 {
        0 => base,
        1 => Color::new(0.30, 0.52, 1.00, a),
        2 => Color::new(0.95, 0.45, 0.65, a),
        3 => Color::new(0.30, 0.78, 0.45, a),
        4 => Color::new(0.95, 0.72, 0.28, a),
        5 => Color::new(0.90, 0.35, 0.30, a),
        6 => Color::new(0.55, 0.45, 0.85, a),
        _ => Color::new(0.78, 0.80, 0.84, a),
    }
}
