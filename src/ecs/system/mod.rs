pub mod behaviours;
pub mod maintenance;
pub mod rendering;

pub use behaviours::*;
pub use maintenance::*;
pub use rendering::*;

use crate::math::*;
use ggez::graphics;

pub fn render_sprite(ctx: &mut ggez::Context, sprite: &graphics::Image, pos: &Vec2f, size: &Size2f) {
    let scale = Vec2f::new(size.width / sprite.width() as f32, size.height / sprite.height() as f32);

    let param = graphics::DrawParam::default()
        .scale(scale)
        .dest((pos.clone() - Vec2f::new(size.width * 0.5, size.height * 0.5)).to_point());
    graphics::draw(ctx, sprite, param).unwrap();
}

fn render_circle(ctx: &mut ggez::Context, pos: &Point2f, radius: f32, color: u32, mode: graphics::DrawMode) {
    let color = graphics::Color::from_rgba_u32(color);
    let circle = graphics::Mesh::new_circle(ctx, mode, Point2f::zero(), radius, 0.5, color).unwrap();
    let param = graphics::DrawParam::default().dest(pos.clone());
    ggez::graphics::draw(ctx, &circle, param).unwrap();
}

pub fn render_fill_circle(ctx: &mut ggez::Context, pos: &Point2f, radius: f32, color: u32) {
    render_circle(ctx, pos, radius, color, graphics::DrawMode::fill());
}

pub fn render_stroke_circle(ctx: &mut ggez::Context, pos: &Point2f, radius: f32, width: f32, color: u32) {
    render_circle(ctx, pos, radius, color, graphics::DrawMode::stroke(width));
}

pub fn render_line(ctx: &mut ggez::Context, points: &[Point2f], width: f32, color: u32) {
    let color = graphics::Color::from_rgba_u32(color);
    let mesh = graphics::Mesh::new_line(ctx, points, width, color).unwrap();
    ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::default()).unwrap();
}

pub fn render_polygon(ctx: &mut ggez::Context, points: &[Point2f], color: u32) {
    let color = graphics::Color::from_rgba_u32(color);
    let mode = graphics::DrawMode::fill();
    let mesh = graphics::Mesh::new_polygon(ctx, mode, points, color).unwrap();
    ggez::graphics::draw(ctx, &mesh, graphics::DrawParam::default()).unwrap();
}
