use pico_args::Arguments;

#[cfg(feature = "develop")]
#[macro_export]
macro_rules! build_type {
    (dev: $dev:expr) => {
        $dev
    };
    (prod: $prod:expr) => {};
    (dev: $dev:expr,prod: $prod:expr) => {
        $dev
    };
}

#[cfg(not(feature = "develop"))]
#[macro_export]
macro_rules! build_type {
    (dev: $dev:expr) => {};
    (prod: $prod:expr) => {
        $prod
    };
    (dev: $dev:expr,prod: $prod:expr) => {
        $prod
    };
}

#[derive(Debug)]
pub struct Config {
    pub skip_menu: bool,
}

impl Config {
    pub fn from_env() -> Self {
        let mut args = Arguments::from_env();

        Self { skip_menu: build_type!(dev: args.contains("--skip-menu"), prod: false) }
    }
}
