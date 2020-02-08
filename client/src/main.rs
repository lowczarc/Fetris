use fetris_protocol::{
    actions, game::Direction, game::GameAction, game::Input, game::PlayerGame,
    game::PlayerMinimalInfos, tetrimino::TetriminoType, ClientRequest, ServerRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
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
                _ => String::new(),
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

fn print_other_player(other_players: &[PlayerMinimalInfos], x: u16, y: u16) {
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

fn print_game(game: &PlayerGame) {
    let matrix = game.matrix();
    let tetrimino = game.current_tetrimino();
    let stocked_tetrimino = game.stocked_tetrimino();
    let pending_tetriminos = game.pending_tetriminos();
    let prediction = if let Some(mut tetrimino_prediction) = game.current_tetrimino().clone() {
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
                || (tetrimino.is_some() && tetrimino.unwrap().check_position(x as i8, y as i8))
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
            } else if prediction.is_some() && prediction.unwrap().check_position(x as i8, y as i8) {
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

        use std::time;
        print_tetrimino_at(pending_tetriminos[j], 23, 8 + (3 * i as u16));
    }
}

fn print_interface(game: &PlayerGame, other_players: &[PlayerMinimalInfos]) {
    print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All,);
    print_game(&game);
    print_other_player(&other_players, 40, 1);
    println!("");
}

fn input_to_action(input: Input) -> GameAction {
    match input {
        Input::Left => GameAction::MoveCurrentTetrimino(Direction::Left),
        Input::Right => GameAction::MoveCurrentTetrimino(Direction::Right),
        Input::FastMove => GameAction::MoveCurrentTetrimino(Direction::FastDown),
        Input::Rotate => GameAction::Rotate,
        Input::StockTetrimino => GameAction::StockTetrimino,
        Input::Acceleration => GameAction::MoveCurrentTetrimino(Direction::Down),
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

    let reader = stream.try_clone().unwrap();
    let _hide_cursor = termion::cursor::HideCursor::from(stdout());

    let _stdout = stdout().into_raw_mode().unwrap();
    let game: Arc<Mutex<Option<PlayerGame>>> = Arc::new(Mutex::new(None));
    let game_board = game.clone();
    let printable_game: Arc<Mutex<Option<PlayerGame>>> = Arc::new(Mutex::new(None));
    let printable_game_request_listener = printable_game.clone();
    let printable_game_keyboard_listener = printable_game.clone();

    thread::spawn(move || {
        let mut other_players = Vec::new();
        println!(
            "{}{}Waiting for other players ...",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
        );
        let mut game =
            if let Ok(ServerRequest::GameReady(game)) = ServerRequest::from_reader(&reader) {
                game
            } else {
                panic!("Invalid first server request");
            };
            {
                let mut board = game_board.lock().unwrap();
                *board = Some(game);
            }

            thread::spawn(move || {
                loop {
                    {
                        let board = printable_game.lock().unwrap();
                        if let Some(board) = board.as_ref() { 
                            print_interface(board, &[]);
                        }
                    }
                    thread::sleep(time::Duration::from_millis(30));
                }
            });

            loop {
                if let Ok(request) = ServerRequest::from_reader(&reader) {
                    if request == ServerRequest::GameOver {
                        let mut board = game_board.lock().unwrap();
                        *board = None;
                        let printable_board = printable_game_request_listener.lock().unwrap();
                        println!("{}-------------", termion::cursor::Goto(4, 12));
                        println!("{}| Game Over |", termion::cursor::Goto(4, 13));
                        println!("{}-------------", termion::cursor::Goto(4, 14));
                        loop {
                            thread::sleep(time::Duration::from_secs(1));
                        }
                    }
                    match request {
                        ServerRequest::PlayerListUpdate(other_players_new) => {
                            other_players = other_players_new;
                        }
                        ServerRequest::MinifiedAction(action) => {
                            let mut board = game_board.lock().unwrap();
                            actions::apply_action(board.as_mut().unwrap(), action);
                            let mut printable_board = printable_game_request_listener.lock().unwrap();
                            *printable_board = board.clone();
                        }
                        _ => {}
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
                {
                    let mut printable_board = printable_game_keyboard_listener.lock().unwrap();
                    let mut game_board_prediction = printable_board.clone();
                    actions::apply_action(game_board_prediction.as_mut().unwrap(), input_to_action(*input));
                    *printable_board = game_board_prediction;
                }
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
