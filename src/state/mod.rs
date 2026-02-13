

pub mod db;

#[derive(Clone)]
pub struct AppState {
    pub db: crate::state::db::Db,
}
