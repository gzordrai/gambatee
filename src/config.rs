use config::{Config as C, ConfigError, File};
use rand::{Rng, seq::IndexedRandom};
use serde::Deserialize;
use serenity::prelude::TypeMapKey;

type Names = Vec<String>;

const CONFIG_FILE: &str = "config.toml";

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub generator: Generator,
    pub drop_rates: DropRates,
    pub channels: Channels,
}

impl TypeMapKey for Config {
    type Value = Config;
}

#[derive(Clone, Debug, Deserialize)]
pub struct Generator {
    pub channel_id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DropRates {
    pub common: f32,
    pub rare: f32,
    pub epic: f32,
    pub legendary: f32,
}

impl DropRates {
    pub fn get_random_drop<'a>(&self, channels: &'a Channels) -> Option<&'a String> {
        let mut rng = rand::rng();
        let num: f32 = rng.random_range(0.0..100.0);

        if num < self.common {
            channels.common.choose(&mut rng)
        } else if num < self.common + self.rare {
            channels.rare.choose(&mut rng)
        } else if num < self.common + self.rare + self.epic {
            channels.epic.choose(&mut rng)
        } else if num < self.common + self.rare + self.epic + self.legendary {
            channels.legendary.choose(&mut rng)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Channels {
    pub common: Names,
    pub rare: Names,
    pub epic: Names,
    pub legendary: Names,
}

pub fn load_config() -> Result<Config, ConfigError> {
    let config = C::builder()
        .add_source(File::with_name(CONFIG_FILE))
        .build()?;

    config.try_deserialize()
}
