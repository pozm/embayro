// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use embayro::{EmbayroState, data::PersistData, anidb::{SearchResult, AnimeEntry}, EmbayroInit, memdata::MemData};
use tauri::{AppHandle, State, Manager, RunEvent};
use tvmaze_api::client::TvShowLookup;

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
#[tauri::command(async)]
async fn show_akas() {

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

#[tauri::command]
fn set_selected(id:u32,query:&str,es : State<'_,EmbayroState>) -> Result<MemData,()> {

	if let Some(memd) = &es.read().mem_data {
		if memd.lookup_id == id {
			return Ok(memd.clone());
		}
	}

	let mut wr = es.write();
	println!("updating selected to {} ({})",query,id);

	wr.mem_data.replace(MemData::new(query,id)).ok_or(())
	
} 
#[tauri::command(async)]
async fn lookup_selected(es : State<'_,EmbayroState>) -> Result<tvmaze_api::json::show::Show,String> {
	println!("Looking up selected");
	let Some(mem_data) = es.read().mem_data.clone() else {
		return Err("(LOOKUP FAIL) No mem data".to_string())
	};

	if mem_data.show_data.is_some() {
		println!("Returning cached data");
		return Ok(mem_data.show_data.clone().unwrap());
	}

	println!("Looking up {}",mem_data.lookup_id);
	let db = {

		let Some(emi) = &es.read().lazy_init else {
			return Err("No db".to_string())
		};
		emi.db_pool.clone()
	};
	let mut db = db.acquire().await.unwrap();
	let tvm = es.read().tvmaze.clone();
	let res = sqlx::query!("SELECT tvbid, def_tvdb_season FROM animelist WHERE anidb_id = ?",mem_data.lookup_id).fetch_one(&mut db).await.or(Err("db lookup fail".to_string()))?;
	let tvbid = res.tvbid.unwrap() as u32;
	let season_count = res.def_tvdb_season.unwrap() as u32;

	println!("{} {}",tvbid,season_count);

	let res= tvm.show_lookup(TvShowLookup::Tvdb(tvbid as i32)).await.map_err(|e|e.to_string())?;
	// let res = res.seasons().await.unwrap();
	// let res = res.get(season_count as usize).unwrap().clone();

	es.write().mem_data.as_mut().unwrap().show_data = res.clone();

	res.ok_or("lookup fail".to_string())
} 
fn main() { 

    tauri::Builder::default()
        .manage(EmbayroState::default())
        .invoke_handler(tauri::generate_handler![get_data,save_data,init,search,search_id,lookup_selected,set_selected])
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
