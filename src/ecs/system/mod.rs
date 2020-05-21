pub mod behaviours;
pub mod maintenance;
pub mod rendering;

use crate::math::*;
use ggez::graphics;

pub fn render_sprite(
    ctx: &mut ggez::Context,
    sprite: &graphics::Image,
    pos: &Vec2f,
    size: &Size2f,
) {
    let scale = Vec2f::new(
        size.width / sprite.width() as f32,
        size.height / sprite.height() as f32,
    );

    let param = graphics::DrawParam::default()
        .scale(scale)
        .dest((pos.clone() - Vec2f::new(size.width * 0.5, size.height * 0.5)).to_point());
    graphics::draw(ctx, sprite, param).unwrap();
}

pub use behaviours::*;
pub use maintenance::*;
pub use rendering::*;
