use crate::assets::LazyImageAsset;
use crate::math::*;
use super::Chunk;

#[derive(Default, Debug)]
pub struct Generator;
impl Generator {
    pub fn generate(&self) -> Chunk {
        Chunk {
            pos: Point2f::zero(),
            background: LazyImageAsset::new("/sprites/map/water.png")
        }
    }
}
