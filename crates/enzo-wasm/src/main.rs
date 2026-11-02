//! Entrada do cliente web. O loop e as regras ficam em `enzo-wasm::app`.

use enzo_wasm::app::App;
use enzo_wasm::window_conf;
use macroquad::prelude::*;

#[macroquad::main(window_conf)]
async fn main() {
    match App::new() {
        Ok(mut app) => app.run().await,
        Err(err) => {
            // O mapa é embutido e testado, mas nunca deixamos o navegador em
            // branco: mostramos o erro em vez de dar panic.
            loop {
                clear_background(Color::from_rgba(20, 20, 32, 255));
                draw_text("EnzoBallZ", 24.0, 56.0, 48.0, WHITE);
                draw_text(
                    format!("Falha ao carregar a fase: {err}"),
                    24.0,
                    96.0,
                    20.0,
                    RED,
                );
                draw_text(
                    "Verifique o console do navegador para mais detalhes.",
                    24.0,
                    126.0,
                    16.0,
                    GRAY,
                );
                next_frame().await;
            }
        }
    }
}
