use serde::{Deserialize, Serialize};

use crate::{
    game::{Direction, Matrix},
    rotation_tetrimino::{rotate_shape, wall_kicks_tests_list},
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TetriminoType {
    I,
    J,
    L,
    O,
    S,
    T,
    Z,
    None,
}

impl TetriminoType {
    pub fn to_blocks(&self) -> Vec<Vec<bool>> {
        match self {
            Self::I => vec![
                vec![false, true, false, false],
                vec![false, true, false, false],
                vec![false, true, false, false],
                vec![false, true, false, false],
            ],
            Self::J => vec![
                vec![true, true, false],
                vec![false, true, false],
                vec![false, true, false],
            ],
            Self::L => vec![
                vec![false, true, false],
                vec![false, true, false],
                vec![true, true, false],
            ],
            Self::O => vec![vec![true, true], vec![true, true]],
            Self::S => vec![
                vec![false, true, false],
                vec![true, true, false],
                vec![true, false, false],
            ],
            Self::T => vec![
                vec![false, true, false],
                vec![true, true, false],
                vec![false, true, false],
            ],
            Self::Z => vec![
                vec![true, false, false],
                vec![true, true, false],
                vec![false, true, false],
            ],
            Self::None => vec![],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tetrimino {
    ttype: TetriminoType,
    rotation: u8,
    position: (i8, i8),
}

impl Tetrimino {
    pub fn new(ttype: TetriminoType) -> Self {
        if ttype == TetriminoType::None {
            panic!("The type none is not a valid tetrimino and should never be constructed");
        }
        Self {
            ttype,
            rotation: 0,
            position: ((10 - ttype.to_blocks().len() as i8) / 2, 22),
        }
    }

    pub fn position(&self) -> (i8, i8) {
        self.position
    }

    pub fn apply_direction(&mut self, direction: Direction) {
        match direction {
            Direction::Left => self.position.0 -= 1,
            Direction::Right => self.position.0 += 1,
            Direction::Down => self.position.1 -= 1,
            Direction::Up => self.position.1 += 1,
            _ => {}
        }
    }

    pub fn rotate(&mut self, matrix: &Matrix, revert_direction: bool) -> bool {
        let mut rotated_tetri = self.clone();
        if revert_direction {
            rotated_tetri.rotation = (self.rotation + 1) % 4;
        } else {
            rotated_tetri.rotation = (self.rotation + 3) % 4;
        }

        let mut success = false;
        for (x, y) in wall_kicks_tests_list(
            self.ttype,
            if revert_direction {
                (rotated_tetri.rotation + 3) % 4
            } else {
                rotated_tetri.rotation
            },
        )
        .iter()
        {
            if revert_direction {
                rotated_tetri.position.0 -= x;
                rotated_tetri.position.1 += y;
            } else {
                rotated_tetri.position.0 += x;
                rotated_tetri.position.1 -= y;
            }
            if rotated_tetri.is_valid(matrix) {
                success = true;
                break;
            }
            if revert_direction {
                rotated_tetri.position.0 += x;
                rotated_tetri.position.1 -= y;
            } else {
                rotated_tetri.position.0 -= x;
                rotated_tetri.position.1 += y;
            }
        }

        if success {
            *self = rotated_tetri;
        }
        success
    }

    pub fn ttype(&self) -> TetriminoType {
        self.ttype
    }

    pub fn check_position(&self, x: i8, y: i8) -> bool {
        let tetri_shape = self.to_blocks();
        let tetri_x = x - self.position.0;
        let tetri_y = self.position.1 - y;

        tetri_x >= 0
            && tetri_y >= 0
            && tetri_x < tetri_shape.len() as i8
            && tetri_y < tetri_shape.len() as i8
            && tetri_shape[tetri_x as usize][tetri_y as usize]
    }

    pub fn is_valid(&self, matrix: &Matrix) -> bool {
        let tetri_shape = self.to_blocks();

        for x in 0..tetri_shape.len() {
            for y in 0..tetri_shape.len() {
                let matrix_pos_x = x as i8 + self.position.0;
                let matrix_pos_y = -(y as i8) + self.position.1;
                if tetri_shape[x][y]
                    && (matrix_pos_x < 0
                        || matrix_pos_x >= matrix[0].len() as i8
                        || matrix_pos_y < 0
                        || matrix_pos_y >= matrix.len() as i8
                        || matrix[matrix_pos_y as usize][matrix_pos_x as usize].is_some())
                {
                    return false;
                }
            }
        }
        true
    }

    pub fn can_move_to(&self, matrix: &Matrix, direction: Direction) -> bool {
        let mut moved_tetrimino = self.clone();
        moved_tetrimino.apply_direction(direction);

        moved_tetrimino.is_valid(matrix)
    }

    pub fn to_blocks(&self) -> Vec<Vec<bool>> {
        rotate_shape(self.rotation, self.ttype.to_blocks())
    }
}
