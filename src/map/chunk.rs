use crate::{
    assets::{AssetManager, LazyImageAsset},
    math::*,
};
use ggez::graphics::{DrawParam, Drawable};

#[derive(Debug)]
pub struct Chunk {
    pub pos: Point2f,
    pub background: LazyImageAsset,
}
impl Chunk {
    const SIZE: f32 = 1000.0;

    pub fn draw(&self, assets: &mut AssetManager, ctx: &mut ggez::Context) {
        let param = DrawParam::new().src([0.0, 0.0, Self::SIZE, Self::SIZE].into()).dest(self.pos);
        match self.background.get(assets, ctx) {
            Ok(bg) => bg.draw(ctx, param).unwrap(),
            Err(err) => log::warn!("Cannot initialize chunk background: {}", err),
        }
    }
}
