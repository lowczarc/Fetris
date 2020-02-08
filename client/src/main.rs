use fetris_protocol::{
    game::Direction, game::Input, game::PlayerMinimalInfos, tetrimino::TetriminoType,
    ClientRequest, ServerRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::thread;
use termion;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

#[derive(Serialize, Deserialize)]
struct Config {
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

    fn to_hashmap(&self) -> HashMap<Key, Input> {
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

fn print_tetrimino_at(tetrimino: TetriminoType, x: u16, y: u16) {
    for j in 0..2 {
        print!("{}  ", termion::cursor::Goto(x, j as u16 + y));
        for i in 0..4 {
            let color = match tetrimino {
                TetriminoType::I => color::Cyan.bg_str().to_string(),
                TetriminoType::J => color::Blue.bg_str().to_string(),
                TetriminoType::L => color::Rgb(255, 173, 0).bg_string(),
                TetriminoType::O => color::Yellow.bg_str().to_string(),
                TetriminoType::S => color::Green.bg_str().to_string(),
                TetriminoType::T => color::Magenta.bg_str().to_string(),
                TetriminoType::Z => color::Red.bg_str().to_string(),
                _ => unreachable!(),
            };
            if i < tetrimino.to_blocks().len()
                && j < tetrimino.to_blocks().len()
                && tetrimino.to_blocks()[i][j]
            {
                print!("{}  {}", color, color::Bg(color::Reset));
            } else {
                print!("  ");
            }
        }
    }
}

fn print_other_player(other_players: Vec<PlayerMinimalInfos>, x: u16, y: u16) {
    for i in 0..other_players.len() {
        print!(
            "{}{}{}{}",
            termion::cursor::Goto(x, i as u16 * 2 + y),
            if other_players[i].dead {
                color::Red.fg_str()
            } else {
                color::White.fg_str()
            },
            other_players[i].name,
            color::Fg(color::Reset),
        );
    }
}

fn main() -> Result<(), std::io::Error> {
    let config = if let Some(config) = Config::from_path("config.toml") {
        config
    } else {
        Config::default()
    };
    let config = config.to_hashmap();

    if env::args().len() != 2 {
        println!("Usage: fetris server_address");
        return Ok(());
    }

    let mut stream = TcpStream::connect(env::args().nth(1).unwrap())?;

    let _stdout = stdout().into_raw_mode().unwrap();
    let _hide_cursor = termion::cursor::HideCursor::from(stdout());

    let reader = stream.try_clone().unwrap();
    thread::spawn(move || {
        println!(
            "{}{}Waiting for other players ...",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        );
        loop {
            if let Ok(request) = ServerRequest::from_reader(&reader) {
                if request == ServerRequest::GameOver {
                    println!("{}-------------", termion::cursor::Goto(4, 12));
                    println!("{}| Game Over |", termion::cursor::Goto(4, 13));
                    println!("{}-------------", termion::cursor::Goto(4, 14));
                    loop {}
                }
                print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All,);
                if let ServerRequest::Action(_, game, other_players) = request {
                    let matrix = game.matrix();
                    let tetrimino = game.current_tetrimino();
                    let stocked_tetrimino = game.stocked_tetrimino();
                    let pending_tetriminos = game.pending_tetriminos();
                    let prediction =
                        if let Some(mut tetrimino_prediction) = game.current_tetrimino().clone() {
                            while tetrimino_prediction.can_move_to(&matrix, Direction::Down) {
                                tetrimino_prediction.apply_direction(Direction::Down);
                            }
                            Some(tetrimino_prediction)
                        } else {
                            None
                        };

                    print!("{}_____________________", termion::cursor::Goto(1, 2));
                    print!("{}▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔", termion::cursor::Goto(1, 25));
                    for j in 0..22 {
                        print!("{}", termion::cursor::Goto(1, j as u16 + 3));
                        let y = 21 - j;
                        for x in 0..10 {
                            if matrix[y][x] != None
                                || (tetrimino.is_some()
                                    && tetrimino.unwrap().check_position(x as i8, y as i8))
                            {
                                let ttype = if let Some(ttype) = matrix[y][x] {
                                    ttype
                                } else {
                                    tetrimino.unwrap().ttype()
                                };

                                let color = match ttype {
                                    TetriminoType::I => color::Cyan.bg_str().to_string(),
                                    TetriminoType::J => color::Blue.bg_str().to_string(),
                                    TetriminoType::L => color::Rgb(255, 173, 0).bg_string(),
                                    TetriminoType::O => color::Yellow.bg_str().to_string(),
                                    TetriminoType::S => color::Green.bg_str().to_string(),
                                    TetriminoType::T => color::Magenta.bg_str().to_string(),
                                    TetriminoType::Z => color::Red.bg_str().to_string(),
                                    TetriminoType::None => color::White.bg_str().to_string(),
                                };
                                print!("{}  {}", color, color::Bg(color::Reset));
                            } else if prediction.is_some()
                                && prediction.unwrap().check_position(x as i8, y as i8)
                            {
                                print!("{}  {}", color::Bg(color::White), color::Bg(color::Reset));
                            } else {
                                print!("  ");
                            }
                        }
                        print!("|");
                    }
                    print!("{}  Hold:", termion::cursor::Goto(23, 1));
                    if stocked_tetrimino != TetriminoType::None {
                        print_tetrimino_at(stocked_tetrimino, 23, 3);
                    }
                    print!("{}  Next:", termion::cursor::Goto(23, 6));
                    for i in 0..pending_tetriminos.len() {
                        let j = pending_tetriminos.len() - 1 - i;

                        print_tetrimino_at(pending_tetriminos[j], 23, 8 + (3 * i as u16));
                    }
                    print_other_player(other_players, 40, 1);
                    println!("");
                }
            } else {
                break;
            }
        }
        reader.shutdown(std::net::Shutdown::Both).unwrap();
    });

    stream
        .write(&ClientRequest::AskForAGame.into_bytes())
        .unwrap();
    let stdin = stdin();
    for c in stdin.keys() {
        let c = c.unwrap();
        if c == Key::Ctrl('c') {
            println!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
            stream.shutdown(std::net::Shutdown::Both).unwrap();
            break;
        }
        if let Some(input) = config.get(&c) {
            if stream
                .write(&ClientRequest::Input(*input).into_bytes())
                .is_err()
            {
                break;
            }
        }
        if stream.peer_addr().is_err() {
            break;
        }
    }
    Ok(())
}
