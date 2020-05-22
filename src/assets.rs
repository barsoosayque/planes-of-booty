use ggez::graphics::{FilterMode, Image};
use log::debug;
use std::{any::Any, collections::BTreeMap, sync::Arc};

#[derive(Default)]
pub struct AssetManager(BTreeMap<String, Arc<dyn Any + Send + Sync>>);

impl AssetManager {
    pub fn get<A: Asset + 'static>(&mut self, key: &str, ctx: &mut A::Context) -> anyhow::Result<Arc<A>> {
        if let Some(asset) = self.0.get(key) {
            Ok(asset.to_owned().downcast::<A>().unwrap())
        } else {
            let asset = Arc::new(A::load(key, ctx)?);
            self.0.insert(key.to_owned(), asset.clone());
            Ok(asset)
        }
    }
}

pub trait Asset: Sized + Send + Sync {
    type Context;
    fn load(key: &str, ctx: &mut Self::Context) -> anyhow::Result<Self>;
}

#[derive(Debug, Clone)]
pub struct ImageAsset(pub Image);

impl Asset for ImageAsset {
    type Context = ggez::Context;

    fn load(key: &str, ctx: &mut Self::Context) -> anyhow::Result<Self> {
        debug!("Loading image asset {:?}", key);
        let mut img = Image::new(ctx, key)?;
        img.set_filter(FilterMode::Linear);
        Ok(ImageAsset(img).into())
    }
}
