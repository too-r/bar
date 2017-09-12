//Structs to parse a toml config file to.
use std::fs::{DirBuilder, File, OpenOptions};
use std::io::prelude::*;
use std::env::home_dir;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub colours: Colours,
    pub general: General,
    pub placeholders: Placeholders,
    pub executables: Executables,
}

#[derive(Deserialize)]
pub struct Executables {
    pub workspace: String,
    pub volume: String,
}

//"Physical" properties of the bar
#[derive(Deserialize)]
pub struct General {
    pub height: i64,
    pub font: String,
    pub icon_font: String,
    pub ws_icons: String,
    pub underline_height: i64,
    pub update_icon: String,
    pub power_icon: String,
}

//Bar segments
#[derive(Deserialize)]
pub struct Placeholders {
    pub workspace: String,
    pub general: String,
    pub power: String,
    pub clock: String,
    pub volume: String,
    pub updates: String,
    pub music: String,
}

//The colours that we want to pass to Lemonbar
#[derive(Deserialize)]
pub struct Colours {
    pub bg_col: String,
    pub bg_sec: String,
    pub fg_col: String,
    pub fg_sec: String,
    pub hl_col: String,
}

pub fn parse_config() -> Config {
    let home_path = home_dir().unwrap();

    let config = "config/lemonhelper/config.toml";

    let home_str = home_path.to_str().unwrap();
    let cfg_path = format!("{}/.{}", home_str, config); //Format the path to the config file as the home directory + the specified config directory

    let mut buf = String::new(); //Create a new string to read config into
    let mut file = File::open(&cfg_path).unwrap();

    file.read_to_string(&mut buf).unwrap();

    toml::from_str(&buf).unwrap()
}
