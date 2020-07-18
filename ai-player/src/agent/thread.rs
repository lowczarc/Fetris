use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
    sync::{Arc, Mutex},
    thread, time,
};

use crate::agent::simulate_action::{place_tetrimino, simulate_action};
use fetris_protocol::{
    game::{Direction, GameAction, Input, Matrix, PlayerGame},
    tetrimino::{Tetrimino, TetriminoType},
    ClientRequest,
};

fn input_to_action(input: Input) -> GameAction {
    match input {
        Input::Left => GameAction::MoveCurrentTetrimino(Direction::Left),
        Input::Right => GameAction::MoveCurrentTetrimino(Direction::Right),
        Input::FastMove => GameAction::MoveCurrentTetrimino(Direction::FastDown),
        Input::Rotate => GameAction::Rotate,
        Input::StockTetrimino => GameAction::StockTetrimino,
        Input::Acceleration => GameAction::MoveCurrentTetrimino(Direction::Down),
        Input::Fall => panic!("Unexpected Fall Input"),
    }
}

pub fn agent_thread(mut stream: TcpStream, game_board: Arc<Mutex<Option<PlayerGame>>>) {
    loop {
        thread::sleep(time::Duration::from_millis(200));
        let mut input_to_do = Vec::new();
        if let Some(game) = &*game_board.lock().unwrap() {
            let mut hashmap: HashMap<(Tetrimino, bool), Vec<Input>> = HashMap::new();
            let matrix = game.matrix();
            if let Some(tetrimino) = game.current_tetrimino() {
                if game.stocked_tetrimino() == TetriminoType::None {
                    stream
                        .write(&ClientRequest::Input(Input::StockTetrimino).into_bytes())
                        .unwrap();
                } else {
                    let stocked_tetrimino = game.stocked_tetrimino();

                    find_all_possible_positions(&mut hashmap, matrix, &tetrimino, Vec::new());

                    find_all_possible_positions(
                        &mut hashmap,
                        matrix,
                        &Tetrimino::new(stocked_tetrimino),
                        vec![Input::StockTetrimino],
                    );

                    let mut placed_tetrimino: Vec<_> = hashmap
                        .keys()
                        .filter(|x| x.1)
                        .map(|x| (x.0, place_tetrimino(&x.0, matrix)))
                        .collect();

                    placed_tetrimino.sort_by(|a, b| score_matrix(&b.1).cmp(&score_matrix(&a.1)));

                    println!("SCORE: {}", score_matrix(&placed_tetrimino[0].1));

                    input_to_do = hashmap
                        .get(&(placed_tetrimino[0].0, true))
                        .unwrap()
                        .iter()
                        .map(|x| x.clone())
                        .collect();
                }
            } else {
                stream
                    .write(&ClientRequest::Input(Input::Fall).into_bytes())
                    .unwrap();
                thread::sleep(time::Duration::from_millis(200));
            }
        }
        for i in input_to_do.iter() {
            stream
                .write(&ClientRequest::Input(*i).into_bytes())
                .unwrap();
            thread::sleep(time::Duration::from_millis(40));
        }
    }
}

pub fn find_all_possible_positions(
    hashmap: &mut HashMap<(Tetrimino, bool), Vec<Input>>,
    matrix: &Matrix,
    tetrimino: &Tetrimino,
    current_inputs: Vec<Input>,
) {
    for input in [
        Input::FastMove,
        Input::Acceleration,
        Input::Rotate,
        Input::Left,
        Input::Right,
    ]
    .iter()
    {
        let mut updated_inputs = current_inputs.clone();
        updated_inputs.push(*input);
        if let Ok(key) = simulate_action(matrix, tetrimino, input_to_action(*input)) {
            if !hashmap.contains_key(&key) {
                hashmap.insert(key, updated_inputs.clone());
                if !key.1 {
                    find_all_possible_positions(hashmap, matrix, &key.0, updated_inputs);
                }
            }
        }
    }
}

pub fn score_matrix(matrix: &Matrix) -> i32 {
    score_contact_surface(matrix) - (2 * get_height(matrix))
}

pub fn get_height(matrix: &Matrix) -> i32 {
    for y in 0..matrix.len() {
        let j = matrix.len() - 1 - y;
        for x in 0..matrix[j].len() {
            if matrix[j][x].is_some() {
                return j as i32;
            }
        }
    }
    return 0;
}

pub fn score_contact_surface(matrix: &Matrix) -> i32 {
    let mut score = 0;
    for y in 0..matrix.len() {
        for x in 0..matrix[y].len() {
            if y != 0 && matrix[y - 1][x].is_some() != matrix[y][x].is_some() {
                score -= 1;
            }
            if y != matrix.len() - 1 && matrix[y + 1][x].is_some() != matrix[y][x].is_some() {
                score -= 1;
            }
            if x != 0 && matrix[y][x - 1].is_some() != matrix[y][x].is_some() {
                score -= 1;
            }
            if x != matrix[y].len() - 1 && matrix[y][x + 1].is_some() != matrix[y][x].is_some() {
                score -= 1;
            }
        }
    }
    score
}
