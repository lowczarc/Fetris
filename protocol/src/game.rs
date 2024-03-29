use serde::{Deserialize, Serialize};

use crate::{
    tetrimino::{Tetrimino, TetriminoType},
    tetrimino_bag::TetriminoBag,
};

pub type Matrix = [[Option<TetriminoType>; 10]; 32];

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerMinimalInfos {
    pub name: String,
    pub dead: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PlayerGame {
    name: String,
    matrix: Matrix,
    current_tetrimino: Option<Tetrimino>,
    stocked_tetrimino: TetriminoType,
    pending_tetriminos: Vec<TetriminoType>,
    bag: TetriminoBag,
}

impl PlayerGame {
    pub fn new(name: String) -> Self {
        let mut bag = TetriminoBag::new();
        let pending_tetriminos = vec![
            bag.choose_a_tetrimino(),
            bag.choose_a_tetrimino(),
            bag.choose_a_tetrimino(),
            bag.choose_a_tetrimino(),
            bag.choose_a_tetrimino(),
            bag.choose_a_tetrimino(),
        ];
        Self {
            name,
            matrix: [[None; 10]; 32],
            current_tetrimino: None,
            stocked_tetrimino: TetriminoType::None,
            pending_tetriminos,
            bag,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    pub fn current_tetrimino(&self) -> Option<Tetrimino> {
        self.current_tetrimino
    }

    pub fn stocked_tetrimino(&self) -> TetriminoType {
        self.stocked_tetrimino
    }

    pub fn pending_tetriminos(&self) -> Vec<TetriminoType> {
        self.pending_tetriminos.clone()
    }

    pub fn stock_current_tetrimino(&mut self) {
        if let Some(current_tetrimino) = self.current_tetrimino {
            let tmp_tetrimino = self.stocked_tetrimino;
            self.stocked_tetrimino = current_tetrimino.ttype();
            self.change_current_tetrimino(tmp_tetrimino);
        }
    }

    pub fn current_tetrimino_mut(&mut self) -> &mut Option<Tetrimino> {
        &mut self.current_tetrimino
    }

    pub fn change_current_tetrimino(&mut self, ttype: TetriminoType) {
        if ttype == TetriminoType::None {
            self.current_tetrimino = None;
        } else {
            self.current_tetrimino = Some(Tetrimino::new(ttype));
        }
    }

    pub fn new_tetrimino(&mut self) -> TetriminoType {
        let added_tetrimino = self.bag.choose_a_tetrimino();

        self.change_tetrimino_add_pending(added_tetrimino);

        added_tetrimino
    }

    pub fn change_tetrimino_add_pending(&mut self, added_tetrimino: TetriminoType) {
        let tetrimino = self.pending_tetriminos.pop().unwrap();

        self.pending_tetriminos.insert(0, added_tetrimino);
        self.change_current_tetrimino(tetrimino);
    }

    pub fn is_line_complete(&self, y: usize) -> bool {
        for x in 0..self.matrix[0].len() {
            if self.matrix[y][x].is_none() {
                return false;
            }
        }
        true
    }

    pub fn remove_complete_lines(&mut self) -> Vec<u8> {
        let mut line_to_remove = Vec::new();
        for y in 0..self.matrix.len() {
            if self.is_line_complete(y) {
                line_to_remove.push(y as u8);
            }
        }

        for line in line_to_remove.iter().rev() {
            for y in (*line as usize)..self.matrix.len() - 1 {
                self.matrix[y] = self.matrix[y + 1];
            }
            self.matrix[self.matrix.len() - 1] = [None; 10];
        }

        line_to_remove
    }

    pub fn place_current_tetrimino(&mut self) -> Vec<u8> {
        if let Some(tetrimino) = self.current_tetrimino {
            let tetri_shape = tetrimino.to_blocks();

            for x in 0..tetri_shape.len() {
                for y in 0..tetri_shape.len() {
                    let position = tetrimino.position();
                    let matrix_pos_x = x as i8 + position.0;
                    let matrix_pos_y = -(y as i8) + position.1;
                    if tetri_shape[x][y] {
                        self.matrix[matrix_pos_y as usize][matrix_pos_x as usize] =
                            Some(tetrimino.ttype());
                    }
                }
            }
        }
        self.current_tetrimino = None;
        self.remove_complete_lines()
    }

    pub fn add_garbage(&mut self, hole: usize) {
        if let Some(tetrimino) = self.current_tetrimino.as_mut() {
            tetrimino.apply_direction(Direction::Up);
        }
        let mut new_row = [Some(TetriminoType::None); 10];
        new_row[hole] = None;

        for y in 0..31 {
            let y = 31 - y;

            self.matrix[y] = self.matrix[y - 1];
        }

        self.matrix[0] = new_row;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    Left,
    Right,
    Down,
    Up,
    FastDown,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum GameAction {
    MoveCurrentTetrimino(Direction),
    PlaceCurrentTetrimino,
    Rotate(bool),
    NewTetrimino(TetriminoType),
    GetGarbage(u32, usize),
    StockTetrimino,
    Fall,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Input {
    Left,
    Right,
    FastMove,
    Rotate,
    RotateRevert,
    StockTetrimino,
    Acceleration,
    Fall,
}
