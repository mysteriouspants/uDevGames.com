use std::sync::RwLock;

use crate::db::DbPool;


pub struct State {
    pub db_pool: RwLock<DbPool>,
}