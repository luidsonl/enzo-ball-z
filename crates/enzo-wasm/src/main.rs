use macroquad::prelude::*;

#[macroquad::main("EnzoBallZ")]
async fn main() {
    loop {
        clear_background(Color::from_rgba(20, 20, 32, 255));
        draw_text("EnzoBallZ", 24.0, 48.0, 48.0, WHITE);
        next_frame().await;
    }
}
