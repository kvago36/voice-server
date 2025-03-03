use libsql::Connection;

pub struct State {
    pub pool: Connection,
    pub token: String,
    pub folder_id: String,
}

impl State {
    pub fn new(pool: Connection, token: String, folder_id: String) -> Self {
        State {
            pool,
            token,
            folder_id,
        }
    }
}
