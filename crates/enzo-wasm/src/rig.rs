//! Manifest-driven humanoid rig: per-state clip/frame resolution and layered composition.
//!
//! Rigs are loaded from `assets/entities/<id>/rig.json` with aligned layer textures.
//! Missing manifests/layers degrade gracefully to legacy sheet rendering.

use std::collections::HashMap;

use enzo_core::entities::VisualProfile;
use macroquad::prelude::*;
use serde::Deserialize;

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
        let manifest = load_manifest(&rig_path)
            .await
            .unwrap_or_else(|| RigManifest {
                id: entity_id.to_owned(),
                image: "sheet.png".to_owned(),
                cell: RigCell { w: 32.0, h: 32.0 },
                anchor: Some(RigAnchor { x: 16.0, y: 30.0 }),
                frames: HashMap::new(),
                clips: HashMap::new(),
                fallbacks: HashMap::new(),
                layers: vec![],
            });
        Self {
            manifest,
            sheet: fallback_sheet,
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

    pub fn has_layers(&self) -> bool {
        self.base.is_some() && self.hair.is_some() && self.outfit_top.is_some()
    }

    pub fn frame_rect(&self, clip_name: &str, time: f32) -> Option<Rect> {
        let frame_name = self.resolve_frame_name(clip_name, time)?;
        self.manifest.frames.get(&frame_name).map(Rect::from)
    }

    fn resolve_frame_name(&self, clip_name: &str, time: f32) -> Option<String> {
        if let Some(name) = self.resolve_clip_frame(clip_name, time) {
            return Some(name);
        }
        if let Some(fallbacks) = self.manifest.fallbacks.get(clip_name) {
            for fb in fallbacks {
                if let Some(name) = self.resolve_clip_frame(fb, time) {
                    return Some(name);
                }
            }
        }
        // generic fallback
        for fb in ["idle", "walk"] {
            if let Some(name) = self.resolve_clip_frame(fb, time) {
                return Some(name);
            }
        }
        self.manifest.frames.keys().next().cloned()
    }

    fn resolve_clip_frame(&self, clip_name: &str, time: f32) -> Option<String> {
        let clip = self.manifest.clips.get(clip_name)?;
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
        clip.frames.get(idx).cloned()
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
            // legacy fallback: draw full sheet frame 0
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
