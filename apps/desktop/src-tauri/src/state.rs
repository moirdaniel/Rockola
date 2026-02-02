use std::path::PathBuf;
use core_db::Db;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub migrations_dir: PathBuf,
    pub media_port: u16,
}