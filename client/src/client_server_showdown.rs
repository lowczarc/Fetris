use fetris_protocol::{
    actions,
    game::{GameAction, PlayerGame},
};

pub struct ActionsQueues(Vec<GameAction>, Vec<GameAction>);

pub enum ShowDownResult {
    NeedReSynchronize,
    ServerLate,
    Synchronized,
}
impl ActionsQueues {
    pub fn new() -> Self {
        Self(vec![], vec![])
    }

    fn are_client_server_synchronized(&self) -> ShowDownResult {
        let Self(client_queue, server_queue) = self;
        if server_queue.len() > client_queue.len() {
            return ShowDownResult::NeedReSynchronize;
        }

        for i in 0..server_queue.len() {
            let i = server_queue.len() - i - 1;
            if let GameAction::NewTetrimino(_) = server_queue.get(i).unwrap() {
                if let GameAction::NewTetrimino(_) = client_queue.get(i).unwrap() {
                } else {
                    return ShowDownResult::NeedReSynchronize;
                }
            } else if server_queue.get(i).unwrap() != client_queue.get(i).unwrap() {
                return ShowDownResult::NeedReSynchronize;
            }
        }
        if server_queue.len() != client_queue.len() {
            return ShowDownResult::ServerLate;
        }
        ShowDownResult::Synchronized
    }

    pub fn push_client_action(&mut self, action: GameAction) {
        self.0.insert(0, action);
    }

    pub fn push_server_action(&mut self, action: GameAction) {
        if let GameAction::GetGarbage(nb, hole_position) = &action {
            let client_server_synchronization = self.client_server_synchronization();
            self.0.push(action.clone());
        }
        self.1.insert(0, action);
    }

    fn client_server_synchronization(&mut self) -> ShowDownResult {
        let client_server_synchronization = self.are_client_server_synchronized();
        match client_server_synchronization {
            ShowDownResult::NeedReSynchronize => {
                panic!(format!(
                    "Synchronization error ({:?}), ({:?})",
                    self.0, self.1
                ));
            }
            ShowDownResult::NeedReSynchronize | ShowDownResult::Synchronized => {
                self.0 = vec![];
                self.1 = vec![];
            }
            ShowDownResult::ServerLate => {
                self.0.truncate(self.0.len() - self.1.len());
                self.1 = vec![];
            }
        }
        client_server_synchronization
    }

    pub fn client_board_prediction(
        &mut self,
        mut board: PlayerGame,
    ) -> (PlayerGame, ShowDownResult) {
        let client_server_synchronization = self.client_server_synchronization();
        for i in 0..self.0.len() {
            let i = self.0.len() - i - 1;
            actions::apply_action(&mut board, self.0.get(i).expect("WTF ?").clone());
        }
        (board, client_server_synchronization)
    }

    pub fn do_last_action_reset_timer(&mut self, board: &PlayerGame) -> bool {
        let client_server_synchronization = self.client_server_synchronization();
        let mut board = board.clone();
        let mut res = false;
        for i in 0..self.0.len() {
            let i = self.0.len() - i - 1;
            if (actions::apply_action(&mut board, self.0.get(i).expect("WTF ?").clone())
                != Err(actions::ApplyActionError::InvalidActionNoResetTimer))
            {
                res = true
            }
        }
        res
    }
}
