//! Render do mundo a partir do snapshot.
//!
//! O render nunca muta o mundo. Posições são interpoladas entre o snapshot
//! anterior e o atual usando `alpha`; estados (pulo, dash, combo, i-frames)
//! vêm do snapshot atual.
//!
//! Divisão interna:
//! - [`poses`] escolhe frames de animação a partir do estado visual;
//! - [`background`] desenha céu, sol, nuvens e montanhas;
//! - [`world`] desenha tiles, entidades, projéteis e efeitos.

mod background;
mod poses;
mod world;

pub use background::{draw_background, Background};
pub use poses::{enzo_frame, item_frame, slime_frame, PlayerPose};
pub use world::{draw_world, RenderContext};
