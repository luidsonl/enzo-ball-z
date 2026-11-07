//! Cliente web do EnzoBallZ.
//!
//! A simulação vive em `enzo-core`; este crate só traduz teclado, desenha o
//! snapshot, toca efeitos visuais e persiste a progressão. Nenhuma regra de jogo
//! é decidida aqui.

pub mod app;
pub mod camera;
pub mod convert;
pub mod effects;
pub mod hud;
pub mod i18n;
pub mod input;
pub mod menu;
pub mod pointer;
pub mod render;
pub mod save;
pub mod sprites;
pub mod tiles;
pub mod ui;
pub mod ui_bridge;

use macroquad::prelude::Conf;

/// Configuração da janela/canvas. Em WASM o tamanho é ditado pelo `<canvas>`.
pub fn window_conf() -> Conf {
    Conf {
        window_title: "EnzoBallZ".to_owned(),
        window_width: 1280,
        window_height: 720,
        high_dpi: true,
        sample_count: 1,
        window_resizable: true,
        ..Default::default()
    }
}
