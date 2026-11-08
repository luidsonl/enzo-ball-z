//! Lazy texture asset cache for WASM/web builds.
//!
//! Textures are loaded on demand with in-flight deduplication.
//! [`pump_assets`] polls in-flight loads without blocking the render loop,
//! so frames keep rendering with placeholders while downloads complete.
//! Single-threaded (macroquad main thread).

use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use macroquad::prelude::*;

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

#[derive(Clone)]
enum LoadState {
    Ready(Texture2D),
    Failed,
}

struct PendingLoad {
    path: String,
    fut: BoxFut<Option<Texture2D>>,
}

#[derive(Default)]
struct StoreInner {
    textures: HashMap<String, LoadState>,
    queued: HashSet<String>,
    queue: VecDeque<String>,
    in_flight: Vec<PendingLoad>,
    max_in_flight: usize,
}

impl StoreInner {
    fn new() -> Self {
        Self {
            max_in_flight: 2,
            ..Default::default()
        }
    }
}

#[derive(Clone)]
pub struct AssetStore {
    inner: Rc<RefCell<StoreInner>>,
}

impl Default for AssetStore {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(StoreInner::new())),
        }
    }
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
            if inner.textures.contains_key(path) || inner.queued.contains(path) {
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
        let inner = self.inner.borrow();
        inner.queue.len() + inner.in_flight.len()
    }

    fn start_queued(&self) {
        let mut inner = self.inner.borrow_mut();
        while inner.in_flight.len() < inner.max_in_flight {
            let Some(path) = inner.queue.pop_front() else {
                break;
            };
            inner.queued.remove(&path);
            let p = path.clone();
            inner.in_flight.push(PendingLoad {
                path,
                fut: Box::pin(load_texture_async(p)),
            });
        }
    }

    fn poll_in_flight(&self) {
        let mut inner = self.inner.borrow_mut();
        let mut i = 0;
        while i < inner.in_flight.len() {
            let ready = {
                let load = &mut inner.in_flight[i];
                match poll_once(&mut load.fut) {
                    Poll::Ready(tex) => Some((load.path.clone(), tex)),
                    Poll::Pending => None,
                }
            };
            if let Some((path, tex)) = ready {
                inner.in_flight.swap_remove(i);
                match tex {
                    Some(t) => {
                        inner.textures.insert(path, LoadState::Ready(t));
                    }
                    None => {
                        inner.textures.insert(path, LoadState::Failed);
                    }
                }
            } else {
                i += 1;
            }
        }
    }
}

/// Polls in-flight downloads/decodes without blocking. Call every frame.
/// Renders continue showing placeholders until textures are ready.
pub fn pump_assets(store: &AssetStore, _max: usize) {
    store.start_queued();
    store.poll_in_flight();
}

pub async fn load_texture_async(path: String) -> Option<Texture2D> {
    let bytes = macroquad::file::load_file(&path).await.ok()?;
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
