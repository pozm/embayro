use serde::{Deserialize, Serialize};
use tvmaze_api::json::show::Show;

#[derive(Serialize, Deserialize,Debug,Clone)]
pub struct MemData {
	pub lookup_name : String,
	pub lookup_id : u32,
	pub show_data : Option<Show>
}

impl MemData {
	pub fn new(name : &str, id: u32) -> Self {
		Self {
			lookup_name : name.to_string(),
			lookup_id : id,
			show_data : None,
		}
	}
}