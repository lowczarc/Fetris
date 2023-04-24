use crate::game::{Direction, GameAction, PlayerGame};

#[derive(PartialEq)]
pub enum ApplyActionError {
    InvalidActionResetTimer,
    InvalidActionNoResetTimer,
}

pub fn apply_action(player: &mut PlayerGame, action: GameAction) -> Result<(), ApplyActionError> {
    match action {
        GameAction::MoveCurrentTetrimino(Direction::FastDown) => {
            let matrix = player.matrix().clone();
            if let Some(tetrimino) = player.current_tetrimino_mut() {
                while tetrimino.can_move_to(&matrix, Direction::Down) {
                    tetrimino.apply_direction(Direction::Down);
                }

                player.place_current_tetrimino();
                Ok(())
            } else {
                Err(ApplyActionError::InvalidActionNoResetTimer)
            }
        }
        GameAction::MoveCurrentTetrimino(direction) => {
            let matrix = player.matrix().clone();
            if let Some(tetrimino) = player.current_tetrimino_mut() {
                if tetrimino.can_move_to(&matrix, direction) {
                    tetrimino.apply_direction(direction);
                    Ok(())
                } else {
                    Err(ApplyActionError::InvalidActionResetTimer)
                }
            } else {
                Err(ApplyActionError::InvalidActionNoResetTimer)
            }
        }
        GameAction::NewTetrimino(added_tetrimino) => {
            player.change_tetrimino_add_pending(added_tetrimino);
            Ok(())
        }
        GameAction::PlaceCurrentTetrimino => {
            player.place_current_tetrimino();
            Ok(())
        }
        GameAction::Rotate(direction) => {
            let matrix = player.matrix().clone();
            if let Some(tetrimino) = player.current_tetrimino_mut() {
                if tetrimino.rotate(&matrix, direction) {
                    Ok(())
                } else {
                    Err(ApplyActionError::InvalidActionResetTimer)
                }
            } else {
                Err(ApplyActionError::InvalidActionNoResetTimer)
            }
        }
        GameAction::StockTetrimino => {
            if player.current_tetrimino().is_some() {
                player.stock_current_tetrimino();
                Ok(())
            } else {
                Err(ApplyActionError::InvalidActionNoResetTimer)
            }
        }
        GameAction::GetGarbage(garbage_to_send, hole_position) => {
            for _ in 0..garbage_to_send {
                player.add_garbage(hole_position);
            }
            Ok(())
        }
        GameAction::Fall => {
            let matrix = player.matrix().clone();
            if let Some(tetrimino) = player.current_tetrimino_mut() {
                if tetrimino.can_move_to(&matrix, Direction::Down) {
                    tetrimino.apply_direction(Direction::Down);
                } else {
                    player.place_current_tetrimino();
                }
                Ok(())
            } else {
                Err(ApplyActionError::InvalidActionResetTimer)
            }
        }
    }
}
