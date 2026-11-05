//! Entrada do cliente web. O loop e as regras ficam em `enzo-wasm::app`.

use enzo_wasm::app::App;
use enzo_wasm::window_conf;

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::new();
    app.run().await;
}
