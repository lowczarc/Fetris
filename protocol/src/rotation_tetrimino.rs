use crate::tetrimino::TetriminoType;

fn rotate_tetrimino_by_one(tetrimino: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut ret = Vec::new();

    for x in 0..tetrimino[0].len() {
        let mut row = Vec::new();
        for y in 0..tetrimino.len() {
            row.insert(0, tetrimino[y][x]);
        }
        ret.push(row);
    }
    ret
}

pub fn rotate_shape(nb_rotate: u8, tetrimino: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    if nb_rotate == 0 {
        return tetrimino;
    }

    rotate_shape((nb_rotate - 1) % 4, rotate_tetrimino_by_one(tetrimino))
}

// Tests wall kicks using SRS: https://tetris.fandom.com/wiki/SRS
pub fn wall_kicks_tests_list(ttype: TetriminoType, rotation: u8) -> [(i8, i8); 5] {
    let b1 = rotation & 1 != 0;
    let b2 = ((4 - rotation) % 4) & 2 != 0;
    let r0 = if b1 { -1 } else { 1 };
    let r1 = if b2 { -1 } else { 1 };
    match (ttype, (b1, b2)) {
        (TetriminoType::I, (false, false)) => [(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        (TetriminoType::I, (true, true)) => [(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        (TetriminoType::I, (false, true)) => [(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        (TetriminoType::I, (true, false)) => [(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        _ => [(0, 0), (-r1, 0), (-r1, r0), (0, -2 * r0), (-r1, -2 * r0)],
    }
}
