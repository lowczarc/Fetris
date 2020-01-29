use rand::{self, Rng};
use serde::{Deserialize, Serialize};

use crate::tetrimino::TetriminoType;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TetriminoBag(Vec<TetriminoType>);

impl TetriminoBag {
    pub fn new() -> Self {
        Self(Self::reseted_list())
    }

    fn reseted_list() -> Vec<TetriminoType> {
        vec![
            TetriminoType::I,
            TetriminoType::J,
            TetriminoType::L,
            TetriminoType::O,
            TetriminoType::S,
            TetriminoType::T,
            TetriminoType::Z,
        ]
    }

    pub fn choose_a_tetrimino(&mut self) -> TetriminoType {
        let random = rand::thread_rng().gen_range(0, self.0.len());
        let tetrimino = self.0.swap_remove(random);

        if self.0.len() == 0 {
            self.0 = Self::reseted_list();
        }
        return tetrimino;
    }
}
