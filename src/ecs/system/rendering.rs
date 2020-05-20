use super::super::component::*;
use crate::math::*;
use ggez::{graphics, Context};
use specs::{Join, ReadStorage, System};

pub struct SpriteRenderSystem<'a>(pub &'a mut Context);
impl SpriteRenderSystem<'_> {
    fn render_sprite(&mut self, sprite: &graphics::Image, pos: &Vec2f, size: &Vec2f) {
        let scale = Vec2f::new(
            size.x / sprite.width() as f32,
            size.y / sprite.height() as f32,
        );

        let param = graphics::DrawParam::default()
            .scale(scale)
            .dest((pos.clone() - Vec2f::new(size.x * 0.5, size.y * 0.5)).to_point());
        graphics::draw(self.0, sprite, param).unwrap();
    }
}
impl<'a> System<'a> for SpriteRenderSystem<'_> {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Movement>,
        ReadStorage<'a, DirectionalSprite>,
        ReadStorage<'a, Sprite>,
    );

    fn run(&mut self, (transforms, movements, dir_sprites, sprites): Self::SystemData) {
        for (transform, movement, sprite) in (&transforms, &movements, &dir_sprites).join() {
            let img = match Direction::from_vec2f(&movement.velocity) {
                Direction::North => &sprite.north.0,
                Direction::East => &sprite.east.0,
                Direction::South => &sprite.south.0,
                Direction::West => &sprite.west.0,
            };

            self.render_sprite(&img, &transform.pos, &Vec2f::new(sprite.width, sprite.height));
        }

        for (transform, sprite) in (&transforms, &sprites).join() {
            self.render_sprite(&sprite.asset.0, &transform.pos, &Vec2f::new(sprite.width, sprite.height));
        }
    }
}
