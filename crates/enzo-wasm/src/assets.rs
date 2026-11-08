//! Lazy texture asset cache for WASM/web builds.
//!
//! Textures are loaded on demand when first requested by rendering code, with
//! in-flight deduplication. Call [`pump_assets`] from the main async loop to
//! progress queued loads. Missing/pending assets draw placeholders without
//! blocking the render loop. Single-threaded (macroquad main thread).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

use macroquad::prelude::*;

#[derive(Clone)]
enum LoadState {
    Loading,
    Ready(Texture2D),
    Failed,
}

#[derive(Default)]
struct StoreInner {
    textures: HashMap<String, LoadState>,
    queued: HashSet<String>,
    queue: VecDeque<String>,
}

#[derive(Clone, Default)]
pub struct AssetStore {
    inner: Rc<RefCell<StoreInner>>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_request(&self, path: &str) -> Option<Texture2D> {
        {
            let inner = self.inner.borrow();
            if let Some(LoadState::Ready(tex)) = inner.textures.get(path) {
                return Some(tex.clone());
            }
            if inner.textures.contains_key(path) {
                return None;
            }
        }
        self.request(path.to_owned());
        None
    }

    pub fn get_optional(&self, path: &str) -> Option<Texture2D> {
        self.get_or_request(path)
    }

    pub fn request(&self, path: String) {
        let mut inner = self.inner.borrow_mut();
        if inner.textures.contains_key(&path) || inner.queued.contains(&path) {
            return;
        }
        inner.queued.insert(path.clone());
        inner.textures.insert(path.clone(), LoadState::Loading);
        inner.queue.push_back(path);
    }

    pub fn peek(&self, path: &str) -> Option<Texture2D> {
        let inner = self.inner.borrow();
        match inner.textures.get(path) {
            Some(LoadState::Ready(tex)) => Some(tex.clone()),
            _ => None,
        }
    }

    pub fn is_ready(&self, path: &str) -> bool {
        matches!(
            self.inner.borrow().textures.get(path),
            Some(LoadState::Ready(_))
        )
    }

    pub fn pending(&self) -> usize {
        self.inner.borrow().queue.len()
    }

    fn pop(&self) -> Option<String> {
        let mut inner = self.inner.borrow_mut();
        let path = inner.queue.pop_front()?;
        inner.queued.remove(&path);
        Some(path)
    }

    fn store_result(&self, path: String, tex: Option<Texture2D>) {
        let mut inner = self.inner.borrow_mut();
        match tex {
            Some(t) => {
                inner.textures.insert(path, LoadState::Ready(t));
            }
            None => {
                inner.textures.insert(path, LoadState::Failed);
            }
        }
    }
}

/// Loads up to `max` queued textures. Safe to call from the async main loop.
pub async fn pump_assets(store: &AssetStore, max: usize) {
    for _ in 0..max {
        let Some(path) = store.pop() else {
            break;
        };
        let result = load_texture_async(&path).await;
        store.store_result(path, result);
    }
}

pub async fn load_texture_async(path: &str) -> Option<Texture2D> {
    let bytes = macroquad::file::load_file(path).await.ok()?;
    let image = Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).ok()?;
    let tex = Texture2D::from_image(&image);
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

/// Draws a texture region if ready; otherwise requests it and draws a placeholder.
pub fn draw_lazy(
    store: &AssetStore,
    path: &str,
    source: Option<Rect>,
    dest: Rect,
    flip_x: bool,
    tint: Color,
) -> bool {
    if let Some(tex) = store.get_or_request(path) {
        draw_texture_ex(
            &tex,
            dest.x,
            dest.y,
            tint,
            DrawTextureParams {
                dest_size: Some(vec2(dest.w, dest.h)),
                source,
                flip_x,
                ..Default::default()
            },
        );
        true
    } else {
        draw_rectangle_lines(
            dest.x,
            dest.y,
            dest.w,
            dest.h,
            1.0,
            Color::from_rgba(180, 160, 255, 40),
        );
        false
    }
}
