use std::{
    sync::{Arc, Mutex},
    thread, time,
};
use termion::color;

use fetris_protocol::{
    game::{Direction, PlayerGame, PlayerMinimalInfos},
    tetrimino::TetriminoType,
};

use crate::client_server_showdown::ActionsQueues;

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

        print_tetrimino_at(pending_tetriminos[j], 23, 8 + (3 * i as u16));
    }
}

pub fn print_interface(game: &PlayerGame, other_players: &[PlayerMinimalInfos]) {
    print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All,);
    print_game(&game);
    print_other_player(&other_players, 40, 1);
    println!("");
}

pub fn launch_print_thread(
    board_mutex: Arc<Mutex<Option<PlayerGame>>>,
    action_queues: Arc<Mutex<ActionsQueues>>,
) {
    thread::spawn(move || loop {
        {
            let mut action_queues = action_queues.lock().unwrap();
            let board = board_mutex.lock().unwrap();
            if let Some(board) = board.as_ref() {
                let client_predicted_board = action_queues.client_board_prediction(board.clone());
                print_interface(&client_predicted_board.0, &[]);
            }
        }
        thread::sleep(time::Duration::from_millis(15));
    });
}
