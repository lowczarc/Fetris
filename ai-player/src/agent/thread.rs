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
        Input::Rotate => GameAction::Rotate(false),
        Input::RotateRevert => GameAction::Rotate(true),
        Input::StockTetrimino => GameAction::StockTetrimino,
        Input::Acceleration => GameAction::MoveCurrentTetrimino(Direction::Down),
        Input::Fall => panic!("Unexpected Fall Input"),
    }
}

pub fn agent_thread(mut stream: TcpStream, game_board: Arc<Mutex<Option<PlayerGame>>>) {
    loop {
        thread::sleep(time::Duration::from_millis(5));
        let mut input_to_do = Vec::new();
        if let Some(game) = &*game_board.lock().unwrap() {
            let matrix = game.matrix();
            if let Some(tetrimino) = game.current_tetrimino() {
                if game.stocked_tetrimino() == TetriminoType::None {
                    stream
                        .write(&ClientRequest::Input(Input::StockTetrimino).into_bytes())
                        .unwrap();
                } else {
                    let stocked_tetrimino = game.stocked_tetrimino();
                    let pending_tetriminos = game.pending_tetriminos();

                    input_to_do = get_best_score_position(
                        tetrimino,
                        stocked_tetrimino,
                        pending_tetriminos,
                        matrix,
                        1,
                    )
                    .1;
                }
            } else {
                stream
                    .write(&ClientRequest::Input(Input::Fall).into_bytes())
                    .unwrap();
                thread::sleep(time::Duration::from_millis(1));
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

pub fn get_best_score_position(
    tetrimino: Tetrimino,
    stocked_tetrimino: TetriminoType,
    pending_tetriminos: Vec<TetriminoType>,
    matrix: &Matrix,
    deep: usize,
) -> (i32, Vec<Input>) {
    let mut hashmap: HashMap<(Tetrimino, bool), Vec<Input>> = HashMap::new();

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
        .map(|x| {
            let placed_simulation = place_tetrimino(&x.0, matrix);
            (x.0, placed_simulation.0, placed_simulation.1)
        })
        .collect();

    if deep == 0 || pending_tetriminos.len() == 0 {
        placed_tetrimino.sort_by(|a, b| {
            let (mut malus_a, mut malus_b) = (0, 0);
            if b.2 > 1 {
                malus_b = (b.2 * b.2 - 1) as i32 * 100
            }
            if a.2 > 1 {
                malus_a = (a.2 * a.2 - 1) as i32 * 100
            }

            (score_matrix(&b.1) + malus_b).cmp(&(score_matrix(&a.1) + malus_a))
        });
        if placed_tetrimino[0].2 > 1 {
            //println!("{}", placed_tetrimino[0].2);
        }
        (
            score_matrix(&placed_tetrimino[0].1),
            hashmap
                .get(&(placed_tetrimino[0].0, true))
                .unwrap()
                .iter()
                .map(|x| x.clone())
                .collect(),
        )
    } else {
        let mut pending_tetriminos = pending_tetriminos;
        let new_tetrimino = Tetrimino::new(pending_tetriminos.pop().unwrap());
        let mut placed_tetrimino: Vec<_> = placed_tetrimino
            .iter()
            .map(|x| {
                let stocked_tetrimino_after_move =
                    if hashmap.get(&(x.0, true)).unwrap().iter().next()
                        == Some(&Input::StockTetrimino)
                    {
                        tetrimino.ttype()
                    } else {
                        stocked_tetrimino
                    };

                let best_score_position = get_best_score_position(
                    new_tetrimino,
                    stocked_tetrimino_after_move,
                    pending_tetriminos.clone(),
                    &x.1,
                    deep - 1,
                );

                let mut malus = 0;
                if x.2 > 1 {
                    malus = (x.2 * x.2 - 1) as i32 * 100;
                    //println!("MALUS !!!");
                }

                (
                    (best_score_position.0 + malus as i32, best_score_position.1),
                    x,
                )
            })
            .collect();

        placed_tetrimino.sort_by(|a, b| (b.0).0.cmp(&(a.0).0));

        (
            (placed_tetrimino[0].0).0,
            hashmap
                .get(&((placed_tetrimino[0].1).0, true))
                .unwrap()
                .iter()
                .map(|x| x.clone())
                .collect(),
        )
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
        Input::Left,
        Input::Right,
        Input::Rotate,
        Input::RotateRevert,
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
