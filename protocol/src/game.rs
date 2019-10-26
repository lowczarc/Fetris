use serde::{Deserialize, Serialize};

type Matrix = [[Option<TetriminoType>; 10]; 32];
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
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
    pub fn to_blocks(&self) -> &'static [&'static [bool]] {
        match self {
            Self::I => &[
                &[false, true, false, false],
                &[false, true, false, false],
                &[false, true, false, false],
                &[false, true, false, false],
            ],
            Self::J => &[
                &[true, true, false],
                &[false, true, false],
                &[false, true, false],
            ],
            Self::L => &[
                &[false, true, false],
                &[false, true, false],
                &[true, true, false],
            ],
            Self::O => &[&[true, true], &[true, true]],
            Self::S => &[
                &[false, true, false],
                &[true, true, false],
                &[true, false, false],
            ],
            Self::T => &[
                &[false, true, false],
                &[true, true, false],
                &[false, true, false],
            ],
            Self::Z => &[
                &[true, false, false],
                &[true, true, false],
                &[false, true, false],
            ],
            Self::None => {
                panic!("The type none is not a valid tetrimino and can't be converted to blocks")
            }
        }
    }

    pub fn wall_kicks_tests(&self, rotation: (bool, bool)) -> [(i8, i8); 5] {
        let r0 = if rotation.0 { -1 } else { 1 };
        let r1 = if rotation.1 { -1 } else { 1 };
        match (self, rotation) {
            (Self::I, (false, false)) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
            (Self::I, (true, true)) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
            (Self::I, (false, true)) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
            (Self::I, (true, false)) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
            _ => [(0, 0), (-r1, 0), (-r1, r0), (0, -2 * r0), (-r1, -2 * r0)],
            _ => unimplemented!(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Tetrimino {
    ttype: TetriminoType,
    rotation: (bool, bool),
    position: (i8, i8),
}

impl Tetrimino {
    pub fn new(ttype: TetriminoType) -> Self {
        if ttype == TetriminoType::None {
            panic!("The type none is not a valid tetrimino and should never be constructed");
        }
        Self {
            ttype,
            rotation: (false, false),
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
            _ => {}
        }
    }

    pub fn rotate(&mut self, matrix: &Matrix) -> bool {
        let mut rotated_tetri = self.clone();
        rotated_tetri.rotation = (!self.rotation.0, !(self.rotation.0 ^ self.rotation.1));

        let mut success = false;
        for (x, y) in self.ttype.wall_kicks_tests(rotated_tetri.rotation).iter() {
            rotated_tetri.position.0 += x;
            rotated_tetri.position.1 -= y;
            println!("{} {}", x, y);
            if rotated_tetri.is_valid(matrix) {
                println!("SUCCESS");
                success = true;
                break;
            }
            rotated_tetri.position.0 -= x;
            rotated_tetri.position.1 += y;
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

    fn is_valid(&self, matrix: &Matrix) -> bool {
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
        let shape = self.ttype.to_blocks();
        let mut new_shape: Vec<Vec<bool>> = Vec::new();

        for i in 0..shape.len() {
            let mut row = Vec::new();
            for j in 0..shape.len() {
                let (i, j) = if self.rotation.1 {
                    (shape.len() - 1 - i, shape.len() - 1 - j)
                } else {
                    (i, j)
                };
                row.push(
                    shape[if self.rotation.0 {
                        shape.len() - 1 - j
                    } else {
                        i
                    }][if self.rotation.0 { i } else { j }],
                );
            }
            new_shape.push(row);
        }
        new_shape
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerGame {
    name: String,
    matrix: Matrix,
    current_tetrimino: Option<Tetrimino>,
    stocked_tetrimino: TetriminoType,
    pending_tetriminos: Vec<TetriminoType>,
}

impl PlayerGame {
    pub fn new(name: String) -> Self {
        Self {
            name,
            matrix: [[None; 10]; 32],
            current_tetrimino: None,
            stocked_tetrimino: TetriminoType::None,
            pending_tetriminos: Vec::new(),
        }
    }

    pub fn matrix(&self) -> &Matrix {
        &self.matrix
    }

    pub fn current_tetrimino(&self) -> Option<Tetrimino> {
        self.current_tetrimino
    }

    pub fn current_tetrimino_mut(&mut self) -> &mut Option<Tetrimino> {
        &mut self.current_tetrimino
    }

    pub fn change_current_tetrimino(&mut self, ttype: TetriminoType) {
        self.current_tetrimino = Some(Tetrimino::new(ttype));
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
            println!("remove line :{}", line);
            for y in (*line as usize)..self.matrix.len() - 1 {
                println!("{} to {}", y + 1, y);
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
                    let matrix_pos_x = x as i8 + tetrimino.position.0;
                    let matrix_pos_y = -(y as i8) + tetrimino.position.1;
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
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Direction {
    Left,
    Right,
    Down,
    FastDown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum GameAction {
    MoveCurrentTetrimino(Direction),
    Rotate,
    NewTetrimino,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Input {
    Left,
    Right,
    FastMove,
    Rotate,
}
