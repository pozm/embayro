// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use embayro::{EmbayroState, data::PersistData, anidb::{AnimeDb, SearchResult, AnimeEntry}, EmbayroInit};
use sqlx::Sqlite;
use tauri::{AppHandle, State, Manager, RunEvent};

#[tauri::command]
fn get_data(es : State<'_,EmbayroState>) -> Result<PersistData,()> {
	if let Some(edata) = &es.read().lazy_init {
		return Ok(edata.persist.clone());
	}
	Err(())
} 
#[tauri::command]
async fn save_data(es : State<'_,EmbayroState>) -> Result<(),()> {
	let Some(init) = &es.write().lazy_init else {
		return Err(());
	};
	init.persist.save_sync().map_err(|_|())

}
#[tauri::command(async)]
async fn init(ah : AppHandle,es : State<'_,EmbayroState>) -> Result<(),()> {
	if es.read().lazy_init.is_some() {
		return Ok(());
	}
	let save_location = PersistData::get_save_location_from_ah(&ah).unwrap().parent().unwrap().to_path_buf();

	let initd = EmbayroInit::new(save_location).await;

	es.write().lazy_init = Some(initd);
	
	Ok(())
}

#[tauri::command]
fn search(q:&str,es : State<'_,EmbayroState>) -> Result<Vec<SearchResult>,()> {
	let Some(db) = &es.read().lazy_init else {
		return Err(())
	};
	db.anidb.search(q).map_err(|_|())
} 
#[tauri::command]
fn search_id(q:u64,es : State<'_,EmbayroState>) -> Result<Vec<AnimeEntry>,()> {
	let Some(db) = &es.read().lazy_init else {
		return Err(())
	};
	db.anidb.get_id(q).map_err(|_|())
} 
#[tauri::command(async)]
fn lookup_id(q:u64,es : State<'_,EmbayroState>) -> Result<Vec<tvmaze_api::responses::show::ShowData>,()> {
	let tvm = &es.read().tvmaze;
	// tvmaze.lookup_show(tvmaze_api::ShowLookup::)
	Err(())
} 
fn main() { 

    tauri::Builder::default()
        .manage(EmbayroState::default())
        .invoke_handler(tauri::generate_handler![get_data,save_data,init,search,search_id])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
		.run(|ah,e| {
			if let RunEvent::WindowEvent { label, event : tauri::WindowEvent::CloseRequested { api, .. }, .. } = e {
				api.prevent_close();
				let state = ah.state::<EmbayroState>();
				let state = state.read();
				if let Some(edata) = &state.lazy_init {
					edata.persist.save_sync().unwrap();
				}
				println!("Closing window {}",label);
				ah.get_window(&label).unwrap().close().unwrap();
			}
		})
}