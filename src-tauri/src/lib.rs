pub mod memdata;
mod nyaa;
pub mod anidb;

use std::path::PathBuf;

use data::PersistData;

pub mod data;

pub struct EmbayroInit {
	pub persist : PersistData,
	pub anidb : anidb::AnimeDb,
	pub db_pool : sqlx::SqlitePool,
}
impl EmbayroInit {
	pub async fn new(save_location : PathBuf) -> Self {

		tokio::fs::create_dir_all(save_location.clone()).await.unwrap();

		// init db
		let db_pool = sqlx::SqlitePool::connect(&format!("sqlite:{}/_.db?mode=rwc",save_location.display())).await.unwrap();
		
		sqlx::migrate!("../migrations").run(&db_pool).await.unwrap();
		
		// init persist data
		let persist = PersistData::load_from(&save_location).await.unwrap_or_else(||PersistData::new(Some(save_location.clone())));
		
		// init anidb
		let mut anidb = anidb::AnimeDb::new(save_location.clone());
		anidb.init(&persist,db_pool.clone()).await.unwrap();

		EmbayroInit {
			persist,
			anidb,
			db_pool,
		}
	}
}
pub struct EmbayroStateInner {
	pub tvmaze : tvmaze_api::client::TvMazeClient,
	pub lazy_init : Option<EmbayroInit>,
	pub mem_data : Option<memdata::MemData>,
}
impl Default for EmbayroStateInner {
    fn default() -> Self {
		let tvmaze = tvmaze_api::client::TvMazeClient::default();
        EmbayroStateInner {
			tvmaze,
			lazy_init : None,
			mem_data : None,
        }
    }
}
pub type EmbayroState = parking_lot::RwLock<EmbayroStateInner>;