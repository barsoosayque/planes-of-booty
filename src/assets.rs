use ggez::graphics::{FilterMode, Image};
use log::debug;
use std::{any::Any, collections::BTreeMap, sync::Arc};

#[derive(Default)]
pub struct AssetManager(BTreeMap<String, Arc<dyn Any + Send + Sync>>, u32);

impl AssetManager {
    pub fn get<A: Asset + 'static>(&mut self, key: &str, ctx: &mut A::Context) -> anyhow::Result<Arc<A>> {
        if let Some(asset) = self.0.get(key) {
            Ok(asset.to_owned().downcast::<A>().unwrap())
        } else {
            let asset = Arc::new(A::load(key, self.1 + 1, ctx)?);
            self.1 += 1;
            self.0.insert(key.to_owned(), asset.clone());
            Ok(asset)
        }
    }
}

pub trait Asset: Sized + Send + Sync {
    type Context;
    fn load(key: &str, id: u32, ctx: &mut Self::Context) -> anyhow::Result<Self>;
    fn id(&self) -> u32;
}

#[derive(Debug, Clone)]
pub struct ImageAsset(u32, Image);
impl std::ops::Deref for ImageAsset {
    type Target = Image;
    fn deref(&self) -> &Self::Target { &self.1 }
}
impl AsRef<Image> for ImageAsset {
    fn as_ref(&self) -> &Image { &self.1 }
}

impl Asset for ImageAsset {
    type Context = ggez::Context;

    fn load(key: &str, id: u32, ctx: &mut Self::Context) -> anyhow::Result<Self> {
        debug!("Loading image asset {:?}", key);
        let mut img = Image::new(ctx, key)?;
        img.set_filter(FilterMode::Linear);
        Ok(ImageAsset(id, img).into())
    }

    fn id(&self) -> u32 { self.0 }
}
