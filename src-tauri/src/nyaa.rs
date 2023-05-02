use std::ops::Deref;

use chrono::{NaiveDateTime};
use scraper::Selector;
use serde::{Deserialize, Serialize};

pub struct NyaaClient {
	pub client : reqwest::Client
}
impl NyaaClient {
	pub fn new() -> Self {
		NyaaClient {
			client : reqwest::Client::new()
		}
	}
}
impl Deref for NyaaClient {
	type Target = reqwest::Client;

	fn deref(&self) -> &Self::Target {
		&self.client
	}
}
#[derive(Debug)]
pub struct ListPage {
	pub animes: Vec<ListPageAnime>,
	// page_count : u32,
}

#[derive(Debug,Clone,Serialize,Deserialize,PartialEq, Eq)]
pub struct ListPageAnime {
	id: u32,
	category : String,
	name : String,
	download_link : String,
	magnet_link : String,
	size : String,
	date : NaiveDateTime,
	seeders : u32,
	leechers : u32,
	completed_downloads : u32,
}

pub struct AnimeViewPage {
	name : String,
	category : String,
	submitter : String,
	info : String,
	size : u64,
	date : NaiveDateTime,
	seeders : u32,
	leechers : u32,
	completed_downloads : u32,
	info_hash : String,
	download_link : String,
	magnet_link : String,

}

impl ListPage {
	pub async fn new_search(query:&str, client:&NyaaClient) -> Self {

		let s_tl = Selector::parse("table.torrent-list > tbody > tr").unwrap();
		let cate_selector  = Selector::parse("td:nth-child(1) > a:not(.comments)").unwrap();
		let name_selector  = Selector::parse("td:nth-child(2) > a:not(.comments)").unwrap();
		let downl_selector  = Selector::parse("td:nth-child(3) > a:first-child").unwrap();
		let magnet_selector  = Selector::parse("td:nth-child(3) > a:last-child").unwrap();
		let size_selector  = Selector::parse("td:nth-child(4)").unwrap();
		let date_selector  = Selector::parse("td:nth-child(5)").unwrap();
		let seeds_selector  = Selector::parse("td:nth-child(6)").unwrap();
		let leeches_selector  = Selector::parse("td:nth-child(7)").unwrap();
		let downloads_selector  = Selector::parse("td:nth-child(8)").unwrap();
	

		let resp = client.get(format!("https://nyaa.si/?f=0&c=1_2&s=seeders&o=desc&q={}",query)).send().await.unwrap().text().await.unwrap();
		let htm = scraper::Html::parse_document(&resp);

		let main_table = htm.select(&s_tl);

		let animes = main_table.map(|child| {

			let cate = child.select(&cate_selector).next().unwrap().value().attr("title").unwrap();
			let name = child.select(&name_selector).next().unwrap().value().attr("title").unwrap();
			let id = child.select(&name_selector).next().unwrap().value().attr("href").unwrap().rsplit_once("/").unwrap().1.parse().unwrap();
			let download_link = child.select(&downl_selector).next().unwrap().value().attr("href").unwrap();
			let magnet_link = child.select(&magnet_selector).next().unwrap().value().attr("href").unwrap();
			let size = child.select(&size_selector).next().unwrap().text().next().unwrap();
			let date_sa = child.select(&date_selector).next().unwrap().value().attr("data-timestamp").unwrap();
			let date = chrono::NaiveDateTime::from_timestamp_opt(date_sa.parse::<i64>().unwrap(), 0).unwrap();
			let seeders = child.select(&seeds_selector).next().unwrap().text().next().unwrap().parse::<u32>().unwrap();
			let leechers = child.select(&leeches_selector).next().unwrap().text().next().unwrap().parse::<u32>().unwrap();
			let completed_downloads = child.select(&downloads_selector).next().unwrap().text().next().unwrap().parse::<u32>().unwrap();

			ListPageAnime {
				id,
				category : cate.to_string(),
				name : name.to_string(),
				date,
				download_link : download_link.to_string(),
				magnet_link : magnet_link.to_string(),
				seeders,
				leechers,
				completed_downloads,
				size : size.to_string()

			}
		}).collect::<Vec<ListPageAnime>>(); 
		Self {
			animes
		}
	}
}


#[cfg(test)]
mod tests {
	#[test]
	fn search_fail() {
		let rt = tokio::runtime::Runtime::new().unwrap();
		let client = super::NyaaClient {
			client : reqwest::Client::new()
		};
		let list = rt.block_on(super::ListPage::new_search(" dasd sadsadsasadasd  dd", &client));
		assert!(list.animes.is_empty())
	}
	#[test]
	fn search_success() {
		let rt = tokio::runtime::Runtime::new().unwrap();
		let client = super::NyaaClient {
			client : reqwest::Client::new()
		};
		let list = rt.block_on(super::ListPage::new_search("one piece", &client));
		assert!(!list.animes.is_empty())
	}
}