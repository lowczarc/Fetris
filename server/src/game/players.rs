use crate::game::pools::PoolState;

#[derive(Clone)]
pub struct Player {
    name: String,
    pool: PoolState,
}

impl Player {
    pub fn new() -> Self {
        Self {
            name: "Anonyme".into(),
            pool: PoolState::None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn pool(&self) -> PoolState {
        self.pool.clone()
    }

    pub fn change_pool(&mut self, pool: PoolState) {
        self.pool = pool;
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
