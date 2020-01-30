use fetris_protocol::{
    game::Direction, game::Input, tetrimino::TetriminoType, ClientRequest, ServerRequest,
};
// use ncurses::*;
use termion;
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{ Read, Write, stdin, stdout };
use std::net::TcpStream;
use std::path::Path;
use std::thread;

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
            left: String::from("KEY_LEFT"),
            right: String::from("KEY_RIGHT"),
            rotate: String::from("^J"),
            instant: String::from("/"),
            accelerate: String::from("KEY_DOWN"),
            stock: String::from("KEY_UP"),
        }
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

fn main() -> Result<(), std::io::Error> {
    let config = if let Some(config) = Config::from_path("config.toml") {
        config
    } else {
        Config::default()
    };

    if env::args().len() != 2 {
        println!("Usage: fetris server_address");
        return Ok(());
    }
    
    // It seems like an unused variable but it actually isn't
    let _hide_cursor = termion::cursor::HideCursor::from(stdout());

    let mut stdout = stdout().into_raw_mode().unwrap();

    let mut stream = TcpStream::connect(env::args().nth(1).unwrap())?;
    let reader = stream.try_clone().unwrap();
    thread::spawn(move || {
        println!(
            "{}{}Waiting for other players ...",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        );
        loop {
            if let Ok(request) = ServerRequest::from_reader(&reader) {
                print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::UntilNewline);
                if let ServerRequest::Action(_, game) = request {
                    let matrix = game.matrix();
                    let tetrimino = game.current_tetrimino();
                    let stocked_tetrimino = game.stocked_tetrimino();
                    let prediction =
                        if let Some(mut tetrimino_prediction) = game.current_tetrimino().clone() {
                            while tetrimino_prediction.can_move_to(&matrix, Direction::Down) {
                                tetrimino_prediction.apply_direction(Direction::Down);
                            }
                            Some(tetrimino_prediction)
                        } else {
                            None
                        };

                    for j in 0..22 {
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
                        print!("|{}", termion::cursor::Goto(1, j as u16 + 2));
                    }
                    print!(
                        "{}{:?}",
                        termion::clear::UntilNewline,
                        tetrimino);
                    if stocked_tetrimino != TetriminoType::None {
                        for y in 0..stocked_tetrimino.to_blocks().len() {
                            print!("{}| ", termion::cursor::Goto(22, y as u16 + 1));
                            for x in 0..stocked_tetrimino.to_blocks().len() {
                                let color = match stocked_tetrimino {
                                    TetriminoType::I => color::Cyan.bg_str().to_string(),
                                    TetriminoType::J => color::Blue.bg_str().to_string(),
                                    TetriminoType::L => color::Rgb(255, 173, 0).bg_string(),
                                    TetriminoType::O => color::Yellow.bg_str().to_string(),
                                    TetriminoType::S => color::Green.bg_str().to_string(),
                                    TetriminoType::T => color::Magenta.bg_str().to_string(),
                                    TetriminoType::Z => color::Red.bg_str().to_string(),
                                    _ => unreachable!(),
                                };
                                if stocked_tetrimino.to_blocks()[x][y] {
                                    print!("{}  {}", color, color::Bg(color::Reset));
                                } else {
                                    print!("  ");
                                }
                            }
                            print!(" |   ");
                        }
                    }
                    println!("");
                }
            } else {
                break;
            }
        }
        println!("Invalid package");
        reader.shutdown(std::net::Shutdown::Both);
    });

    stream.write(&ClientRequest::AskForAGame.into_bytes());
    let stdin = stdin();
    for c in stdin.keys() {
        /*
        let key = if let Some(key) = keyname(c) {
            key
        } else {
            String::from("unknown")
        };
            */

        match c.unwrap() {
            Key::Char('q') => {
                println!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
                break;
            }
            Key::Char('/') => {
                stream.write(&ClientRequest::Input(Input::FastMove).into_bytes());
            }
            Key::Up => {
                stream.write(&ClientRequest::Input(Input::StockTetrimino).into_bytes());
            }
            Key::Down => {
                stream.write(&ClientRequest::Input(Input::Acceleration).into_bytes());
            }
            Key::Left => {
                stream.write(&ClientRequest::Input(Input::Left).into_bytes());
            }
            Key::Right => {
                stream.write(&ClientRequest::Input(Input::Right).into_bytes());
            }
            Key::Char('\n') => {
            stream.write(&ClientRequest::Input(Input::Rotate).into_bytes());
            }
            _ => {}
        }
        /*
        if key == config.left {
            stream.write(&ClientRequest::Input(Input::Left).into_bytes());
            // printw("LEFT");
        } else if key == config.right {
            stream.write(&ClientRequest::Input(Input::Right).into_bytes());
            // printw("RIGHT");
        } else if key == config.rotate {
            stream.write(&ClientRequest::Input(Input::Rotate).into_bytes());
            // printw("ROTATE");
        } else if key == config.instant {
            stream.write(&ClientRequest::Input(Input::FastMove).into_bytes());
            // printw("FAST");
        } else if key == config.accelerate {
            stream.write(&ClientRequest::Input(Input::Acceleration).into_bytes());
            // printw("ACCELERATION");
        } else if key == config.stock {
            stream.write(&ClientRequest::Input(Input::StockTetrimino).into_bytes());
            // printw("FAST");
        } else {
            // printw(&format!("{},{}", c, key));
        }

        */
        if stream.peer_addr().is_err() {
            break;
        }
    }
    Ok(())
}
