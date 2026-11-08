//! Render do mundo a partir do snapshot.
//!
//! - [`entities`] contém animação/desenho de corpos (personagens/itens).
//! - [`skills`] contém projéteis e overlays de ataque.
//! - [`world`] orquestra o desenho.

mod background;
pub mod entities;
pub mod skills;
mod world;

pub use background::{draw_background, Background};
pub use world::{draw_world, RenderContext};

// Compat: reexports usados por testes/código legado.
pub use entities::items::frame as item_frame;
pub use entities::slime::frame as slime_frame;
