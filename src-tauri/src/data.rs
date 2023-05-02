use std::path::PathBuf;

use chrono::Duration;
use serde::{Serialize, Deserialize};
use tauri::api::path::app_data_dir;

#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct PersistData {
    last_list_pull : chrono::DateTime<chrono::Utc>,
    qb_username : String,
    qb_password : String,
    qb_url : String,
    pub save_location : PathBuf,
	first_run : bool,


	#[serde(skip)]
	pub file_save_loc : Option<PathBuf>
}
impl PersistData {
    pub fn get_last_pull(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.last_list_pull
    }

    pub async fn load_from(path: &PathBuf) -> Option<PersistData> {
        let data = tokio::fs::OpenOptions::new()
        .read(true)
        .open(path).
        await.ok()?; 

        let get_std_file = data.into_std().await;
        let mut jsn = tauri::async_runtime::spawn_blocking(|| {
            serde_json::from_reader::<_,Self>(get_std_file)
        }).await.ok()?.ok()?;

		jsn.file_save_loc = Some(path.clone());

        Some(jsn)
    }
    pub async fn save_to(&self, path: &PathBuf) -> anyhow::Result<()> {

		tokio::fs::create_dir_all(path.parent().unwrap()).await?;

        let data = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .await?;
    
        let get_std_file = data.into_std().await;
        let slf = self.clone();
        tauri::async_runtime::spawn_blocking(move || {
            serde_json::to_writer::<_,Self>(get_std_file, &slf)
        }).await??;

		println!("successfully saved to {:?}", path);

        Ok(())
    }
	pub fn get_save_location_from_ah(ah : &tauri::AppHandle) -> Option<PathBuf> {
		let mut data_dir = app_data_dir(ah.config().as_ref())?;
		data_dir.push("persist.json");
		Some(data_dir)
	}
	pub fn save_to_sync(&self, path: &PathBuf) -> anyhow::Result<()> {

		std::fs::create_dir_all(path.parent().unwrap())?;

		let data = std::fs::OpenOptions::new()
		.write(true)
		.create(true)
		.open(path)
		.unwrap();

		serde_json::to_writer::<_,Self>(data, self)?;

		println!("successfully saved to {:?}", path);

		Ok(())
	}

	pub async fn save(&self) -> anyhow::Result<()> {
		if let Some(path) = &self.file_save_loc {
			self.save_to(path).await
		}
		else {
			Err(anyhow::anyhow!("No save location found"))
		}
	}
	pub fn save_sync(&self) -> Result<(), anyhow::Error> {
		if let Some(path) = &self.file_save_loc {
			self.save_to_sync(path)
		}
		else {
			Err(anyhow::anyhow!("No save location found"))
		}
	}
    pub fn new(save_to: Option<PathBuf>) -> PersistData {
        PersistData {
            last_list_pull : chrono::Utc::now() - Duration::days(5),
            qb_username : String::new(),
            qb_password : String::new(),
            qb_url : String::new(),
            save_location : PathBuf::new(),
			first_run:true,
			file_save_loc : save_to
        }
    }
	pub fn setup_complete(&mut self) {
		self.first_run = false;
	}
	pub fn setup_state(&self) -> bool {
		self.first_run
	}
	pub fn get_save_location(&self) -> &PathBuf {
		&self.save_location
	}
	pub fn get_file_location(&self) -> Option<&PathBuf> {
		self.file_save_loc.as_ref()
	}
}