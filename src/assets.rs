use ggez::{graphics::Image, Context};
use log::debug;
use warmy::{Load, Loaded};

pub type Assets = warmy::Store<Context, Key>;
type Storage = warmy::Storage<Context, Key>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key(std::path::PathBuf);

impl<P> std::convert::From<P> for Key where P: AsRef<std::path::Path> {
    fn from(path: P) -> Self {
        Key(path.as_ref().to_owned())
    }
}

impl warmy::key::Key for Key {
    fn prepare_key(self, _root: &std::path::Path) -> Self {
        Self(std::path::Path::new("/").join(self.0))
    }
}

#[derive(Debug, Clone)]
pub struct ImageAsset(pub Image);

impl Load<Context, Key> for ImageAsset {
    type Error = anyhow::Error;

    fn load(
        key: Key,
        _storage: &mut Storage,
        ctx: &mut Context,
    ) -> Result<Loaded<Self, Key>, Self::Error> {
        debug!("Loading image asset {:?}", key.0);
        let img = Image::new(ctx, key.0)?;
        Ok(ImageAsset(img).into())
    }
}
