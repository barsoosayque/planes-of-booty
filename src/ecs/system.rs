use super::{component::*, resource::*};
use crate::math::*;
use ggez::{graphics, Context};
use specs::{Join, Read, ReadStorage, System, WriteStorage};

pub struct SpriteRenderSystem<'a>(pub &'a mut Context);
impl<'a> System<'a> for SpriteRenderSystem<'_> {
    type SystemData = (
        ReadStorage<'a, Transform>,
        ReadStorage<'a, Movement>,
        ReadStorage<'a, DirectionalSprite>,
    );

    fn run(&mut self, (transforms, movements, sprites): Self::SystemData) {
        for (transform, movement, sprite) in (&transforms, &movements, &sprites).join() {
            let img = match movement.direction {
                Direction::North => &sprite.north.0,
                Direction::East => &sprite.east.0,
                Direction::South => &sprite.south.0,
                Direction::West => &sprite.west.0,
            };
            let scale = Vec2f::new(
                sprite.width / img.width() as f32,
                sprite.height / img.height() as f32,
            );

            let param = graphics::DrawParam::default().scale(scale).dest(
                (transform.pos - Vec2f::new(sprite.width * 0.5, sprite.height * 0.5)).to_point(),
            );
            graphics::draw(self.0, img, param).unwrap();
        }
    }
}

pub struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        WriteStorage<'a, Transform>,
        ReadStorage<'a, Movement>,
        Read<'a, DeltaTime>,
    );

    fn run(&mut self, (mut transforms, movements, delta): Self::SystemData) {
        use log::debug;
        for (transform, movement) in (&mut transforms, &movements).join() {
            transform.pos += movement.acc * delta.0.as_secs_f32();
        }
    }
}
