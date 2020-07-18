use fetris_protocol::{
    game::{Direction, GameAction, Matrix},
    tetrimino::Tetrimino,
};

pub enum SimulateActionError {
    ImpossibleAction,
}

pub fn simulate_action(
    matrix: &Matrix,
    tetrimino: &Tetrimino,
    action: GameAction,
) -> Result<(Tetrimino, bool), SimulateActionError> {
    let mut tetrimino = tetrimino.clone();
    match action {
        GameAction::MoveCurrentTetrimino(Direction::FastDown) => {
            while tetrimino.can_move_to(&matrix, Direction::Down) {
                tetrimino.apply_direction(Direction::Down);
            }

            Ok((tetrimino, true))
        }
        GameAction::MoveCurrentTetrimino(direction) => {
            if tetrimino.can_move_to(&matrix, direction) {
                tetrimino.apply_direction(direction);
                Ok((tetrimino, false))
            } else {
                Err(SimulateActionError::ImpossibleAction)
            }
        }
        GameAction::Rotate => {
            if tetrimino.rotate(&matrix) {
                Ok((tetrimino, false))
            } else {
                Err(SimulateActionError::ImpossibleAction)
            }
        }
        GameAction::Fall => {
            if tetrimino.can_move_to(&matrix, Direction::Down) {
                tetrimino.apply_direction(Direction::Down);
                Ok((tetrimino, false))
            } else {
                Ok((tetrimino, true))
            }
        }
        _ => {
            // TODO: Implement StockTetrimino
            Err(SimulateActionError::ImpossibleAction)
        }
    }
}

pub fn is_line_complete(matrix: &Matrix, y: usize) -> bool {
    for x in 0..matrix[0].len() {
        if matrix[y][x].is_none() {
            return false;
        }
    }
    true
}

pub fn remove_complete_lines(matrix: &mut Matrix) {
    let mut line_to_remove = Vec::new();
    for y in 0..matrix.len() {
        if is_line_complete(matrix, y) {
            line_to_remove.push(y as u8);
        }
    }

    for line in line_to_remove.iter().rev() {
        for y in (*line as usize)..matrix.len() - 1 {
            matrix[y] = matrix[y + 1];
        }
        matrix[matrix.len() - 1] = [None; 10];
    }
}

pub fn place_tetrimino(tetrimino: &Tetrimino, matrix: &Matrix) -> Matrix {
    let mut matrix = matrix.clone();
    let tetri_shape = tetrimino.to_blocks();

    for x in 0..tetri_shape.len() {
        for y in 0..tetri_shape.len() {
            let position = tetrimino.position();
            let matrix_pos_x = x as i8 + position.0;
            let matrix_pos_y = -(y as i8) + position.1;
            if tetri_shape[x][y] {
                matrix[matrix_pos_y as usize][matrix_pos_x as usize] = Some(tetrimino.ttype());
            }
        }
    }

    remove_complete_lines(&mut matrix);
    matrix
}
