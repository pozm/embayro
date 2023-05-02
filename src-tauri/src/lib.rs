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
		// init db
		let db_pool = sqlx::SqlitePool::connect(&format!("sqlite:{}/_.db",save_location.display())).await.unwrap();
		
		sqlx::migrate!("../migrations").run(&db_pool).await.unwrap();
		
		// init persist data
		let persist = PersistData::load_from(&save_location).await.unwrap_or_else(||PersistData::new(Some(save_location.clone())));
		
		// init anidb
		let mut anidb = anidb::AnimeDb::new(save_location.clone());
		anidb.init(&persist).await;

		EmbayroInit {
			persist,
			anidb,
			db_pool,
		}
	}
}
pub struct EmbayroStateInner {
	pub tvmaze : tvmaze_api::TvMazeClient,
	pub lazy_init : Option<EmbayroInit>,
}
impl Default for EmbayroStateInner {
    fn default() -> Self {
		let cacher = tvmaze_api::cacher::InMemoryCacher::new(None);
		let tvmaze = tvmaze_api::TvMazeClient::new(Box::new(cacher));
        EmbayroStateInner {
			tvmaze,
			lazy_init : None,
        }
    }
}
pub type EmbayroState = parking_lot::RwLock<EmbayroStateInner>;