use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, path::Path};
use termion::event::Key;

use fetris_protocol::game::Input;

#[derive(Serialize, Deserialize)]
pub struct Config {
    left: String,
    right: String,
    rotate: String,
    instant: String,
    accelerate: String,
    stock: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            left: String::from("Left"),
            right: String::from("Right"),
            rotate: String::from("Enter"),
            instant: String::from("/"),
            accelerate: String::from("Down"),
            stock: String::from("Up"),
        }
    }
}

impl Config {
    fn parse_key(config_key: &str) -> Key {
        match config_key {
            "Left" => Key::Left,
            "Right" => Key::Right,
            "Down" => Key::Down,
            "Up" => Key::Up,
            "Backspace" => Key::Backspace,
            "Enter" => Key::Char('\n'),
            "Space" => Key::Char(' '),
            "Home" => Key::Home,
            "End" => Key::End,
            "PageUp" => Key::PageUp,
            "PageDown" => Key::PageDown,
            "BackTab" => Key::BackTab,
            "Delete" => Key::Delete,
            "Insert" => Key::Insert,
            "Esc" => Key::Esc,
            x if x.len() == 1 => Key::Char(x.chars().next().unwrap()),
            x => panic!(format!("Invalid key in config file: '{}'", x)),
        }
    }

    pub fn to_hashmap(&self) -> HashMap<Key, Input> {
        let mut ret = HashMap::new();

        ret.insert(Self::parse_key(&self.left), Input::Left);
        ret.insert(Self::parse_key(&self.right), Input::Right);
        ret.insert(Self::parse_key(&self.rotate), Input::Rotate);
        ret.insert(Self::parse_key(&self.instant), Input::FastMove);
        ret.insert(Self::parse_key(&self.accelerate), Input::Acceleration);
        ret.insert(Self::parse_key(&self.stock), Input::StockTetrimino);

        ret
    }
}

impl Config {
    pub fn from_path(path: &str) -> Option<Self> {
        let conf_path = Path::new(path);
        let mut conf_file = if let Ok(conf_file) = File::open(conf_path) {
            conf_file
        } else {
            return None;
        };
        let mut conf_str = String::new();
        conf_file.read_to_string(&mut conf_str).unwrap();

        Some(toml::from_str(&conf_str).unwrap())
    }
}
