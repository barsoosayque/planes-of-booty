use crate::shader::ShaderInName;
use gfx::{memory::Pod, pso::buffer::Structure, shade::ConstFormat};
use ggez::graphics::{FilterMode, Image, Shader, WrapMode};
use log::debug;
use std::{any::Any, cell::UnsafeCell, collections::BTreeMap, sync::Arc};

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

    pub fn key_for<A: Asset + 'static>(&self, asset: &A) -> Option<&String> {
        self.0
            .iter()
            .find(|(_, a)| if let Some(a) = a.downcast_ref::<A>() { a.id() == asset.id() } else { false })
            .map(|(key, _)| key)
    }
}

pub trait Asset: Sized + Send + Sync {
    type Context;
    fn load(key: &str, id: u32, ctx: &mut Self::Context) -> anyhow::Result<Self>;
    fn id(&self) -> u32;
}

#[derive(Debug)]
pub struct LazyAsset<A: Asset>(String, UnsafeCell<Option<Arc<A>>>);
impl<A: Asset + 'static> LazyAsset<A> {
    pub fn new(key: &str) -> Self { Self(key.to_owned(), UnsafeCell::new(None)) }

    pub fn try_get<'a>(&'a self) -> Option<&'a A> {
        unsafe { self.1.get().as_ref().and_then(|opt| opt.as_ref()).map(|arc| arc.as_ref()) }
    }

    pub fn get<'a>(&'a self, assets: &mut AssetManager, ctx: &mut A::Context) -> anyhow::Result<&'a A> {
        unsafe {
            match self.1.get().as_ref().and_then(|opt| opt.as_ref()) {
                Some(asset) => Ok(asset),
                None => {
                    self.1.get().replace(Some(assets.get::<A>(&self.0, ctx)?));
                    Ok(self.1.get().as_ref().unwrap().as_ref().unwrap())
                },
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImageAsset(u32, Image);
pub type LazyImageAsset = LazyAsset<ImageAsset>;
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
        img.set_wrap(WrapMode::Tile, WrapMode::Tile);
        Ok(ImageAsset(id, img).into())
    }

    fn id(&self) -> u32 { self.0 }
}

#[derive(Debug, Clone)]
pub struct ShaderAsset<C: Clone + Copy + Pod + Structure<ConstFormat>>(u32, Shader<C>);
impl<C: Copy + Clone + Pod + Structure<ConstFormat>> std::ops::Deref for ShaderAsset<C> {
    type Target = Shader<C>;

    fn deref(&self) -> &Self::Target { &self.1 }
}
impl<C: Copy + Clone + Pod + Structure<ConstFormat>> AsRef<Shader<C>> for ShaderAsset<C> {
    fn as_ref(&self) -> &Shader<C> { &self.1 }
}

impl<C: 'static + Default + ShaderInName + Sync + Send + Copy + Clone + Pod + Structure<ConstFormat>> Asset
    for ShaderAsset<C>
{
    type Context = ggez::Context;

    fn load(key: &str, id: u32, ctx: &mut Self::Context) -> anyhow::Result<Self> {
        // hardcoded for now
        let vert = "/shaders/default.vert";
        let frag = key;
        debug!("Loading shader asset ({}, {})", vert, frag);

        let shader = Shader::new(ctx, vert, frag, C::default(), C::name(), None)?;
        Ok(ShaderAsset(id, shader).into())
    }

    fn id(&self) -> u32 { self.0 }
}
