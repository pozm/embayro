use std::{path::PathBuf, io::{BufReader, Error, ErrorKind}, sync::Arc};

use chrono::Duration;
use futures::{stream::FuturesUnordered, StreamExt};
use quick_xml::events::Event;
use serde::Serialize;
use sqlx::SqlitePool;
use tantivy::{directory::MmapDirectory, schema::{Schema, self, Field}, Index, collector::{Collector, self}, Document, TantivyError, query::{QueryParser, RangeQuery}};
use tokio::{fs::OpenOptions, io::{AsyncWriteExt, AsyncBufRead}, sync::Semaphore};
use tokio_util::{io::StreamReader};

use crate::data::PersistData;

pub struct AnimeDb {
	save_path : PathBuf,

	idf : Field,
	txf : Field,

	index : tantivy::Index,
	indexw : tantivy::IndexWriter,
	reader : tantivy::IndexReader,
}

impl AnimeDb {
	pub fn new(save_path : PathBuf) -> Self {
		std::fs::create_dir_all(save_path.join("anime_db"));
		let tdir = MmapDirectory::open(save_path.join("anime_db")).unwrap();

		// schema

		let mut schema = Schema::builder();
		let idf = schema.add_u64_field("aid", schema::STORED | schema::INDEXED);
		let txf = schema.add_text_field("title", schema::TEXT | schema::STORED);
		
		// indexing

		let index = Index::open_or_create(tdir, schema.build()).unwrap();
		let indexw = index.writer(50_000_000).unwrap(); // 50mb

		let reader = index.reader_builder().reload_policy(tantivy::ReloadPolicy::Manual).try_into().unwrap();
		// return

		AnimeDb {
			save_path,
			idf,
			txf,
			index,
			indexw,
			reader
		}
	}
	pub async fn init(&mut self, perst: &PersistData, db: SqlitePool) -> anyhow::Result<()> {


		if *perst.get_last_pull() + Duration::days(1) > chrono::Utc::now() {
			return Err(anyhow::anyhow!("already pulled today"));
		}

		let x = futures::future::join(self.fetch_master(db),self.fetch_new()).await;

		self.indexw.commit()?;

		x.0?;
		x.1

	}
	pub fn search(&self, query: &str) ->  Result<Vec<SearchResult>,TantivyError> {
		let srch  = self.reader.searcher();
		let queryb = QueryParser::for_index(&self.index, vec![self.txf.clone()]);
		let qq = queryb.parse_query(&query).unwrap();
		let mut res = Vec::new();
		for (score,doc) in srch.search(&qq, &collector::TopDocs::with_limit(10))? {
			let doc = srch.doc(doc)?;
			res.push(SearchResult {
				entry : AnimeEntry {
					id : doc.get_first(self.idf).unwrap().as_u64().unwrap(),
					title : doc.get_first(self.txf).unwrap().as_text().unwrap().to_string(),
				},
				score
			});
		}
		Ok(res)
	}
	pub fn get_id(&self, aid: u64) -> Result<Vec<AnimeEntry>,TantivyError> {
		let srch = self.reader.searcher();
		let qp = RangeQuery::new_u64(self.idf, aid..aid+1);
		let mut res = Vec::new();
		for (_score,doc) in srch.search(&qp, &collector::TopDocs::with_limit(10))? {
			let doc = srch.doc(doc)?;
			res.push(AnimeEntry {
				id : doc.get_first(self.idf).unwrap().as_u64().unwrap(),
				title : doc.get_first(self.txf).unwrap().as_text().unwrap().to_string(),
			});
		}
		Ok(res)
	}
	pub fn save_entry(&self, entry : &AnimeEntry) -> anyhow::Result<u64> {
		// println!("saving entry {:?}", entry);
		if entry.id == 0 {
			println!("{}, {} = 0", entry.id, entry.title);
			return Err(anyhow::anyhow!("entry id is 0"));
		}
		let mut doc = Document::new();
		doc.add_u64(self.idf, entry.id);
		doc.add_text(self.txf, entry.title.clone());
		Ok(self.indexw.add_document(doc)?)
	}
	pub async fn fetch_master(&self, db: SqlitePool) -> anyhow::Result<()> {

		let count = if let Ok(x) = sqlx::query!("SELECT anidb_id FROM animelist WHERE anidb_id=(SELECT max(anidb_id) FROM animelist);"). fetch_one(&db).await {
			x.anidb_id
		} else {
			0
		};

		println!("master already has {count}");
		

		let xml_data = reqwest::get("https://raw.githubusercontent.com/Anime-Lists/anime-lists/master/anime-list.xml").await?.bytes_stream();
		let x2 = xml_data.map(|x| {
			match x {
				Ok(b) => Ok(b),
				Err(e) => Err(Error::new(ErrorKind::Other, e.to_string()))
			}
		});
		let mut r = quick_xml::Reader::from_reader(StreamReader::new(x2));
		r.trim_text(true);

		let mut buf = Vec::new();
		let mut read_text = false;
		let mut this_entry = MasterEntry::default();
		let sema = Arc::new(Semaphore::new(7));
		let mut futurev = Vec::new();
		
		loop {
			let Ok(ev) = r.read_event_into_async(&mut buf).await else {
				continue;
			};
			match ev {
				Event::Start(bs) => {
					match bs.name().as_ref() {
						b"anime" => {
							let dec = r.decoder();
							let Some(aid) = bs.attributes().flatten().find(|x| x.key.as_ref() == b"anidbid") else {
								continue;
							};
							let Some(tvdbid) = bs.attributes().flatten().find(|x| x.key.as_ref() == b"tvdbid") else {
								continue;
							};
							let imdbid = bs.attributes().flatten().find(|x| x.key.as_ref() == b"imdbid");
							let default_tvdb_season = bs.attributes().flatten().find(|x| x.key.as_ref() == b"defaulttvdbseason");
	


							this_entry.aid = dec.decode(&aid.value).unwrap().parse().unwrap();

							if this_entry.aid < count as u32 {
								this_entry = MasterEntry::default();
								continue;
							}

							let Ok(hold) = dec.decode(&tvdbid.value).unwrap().parse::<u32>() else {continue};
							this_entry.tvdbid = hold;
							this_entry.imdbid = imdbid.map(|x| dec.decode(&x.value).unwrap().parse().unwrap_or_default());
							this_entry.defaulttvdbseason = default_tvdb_season.map(|x| dec.decode(&x.value).unwrap().parse().unwrap_or_default());

						},
						b"name" => {
							read_text = true;
						},
						_=>{}
					}
				}
				Event::Text(bs) => {
					if !read_text {
						continue;
					}
					let Ok(ues) = bs.unescape() else {
						continue;
					};
					this_entry.series_title = ues.to_string();
					read_text = false;
				}
				Event::End(bs) => {
					match bs.name().as_ref() {
						b"anime" => {
							if this_entry.aid == 0 {
								continue;
							}
							futurev.push(Self::push_entry(db.clone(), this_entry.clone(),sema.clone()));
							this_entry = MasterEntry::default();
						}
						_=>{}
					}
				}
				Event::Eof => {
					break;
				}
				_=>{}
			}
		}
		println!("waiting for {} futures",futurev.len());
		futures::future::join_all(futurev).await;
		println!("futures are done");
		Ok(())
	}

	async fn push_entry(db: SqlitePool, entry: MasterEntry, sema: Arc<Semaphore>) -> anyhow::Result<()> {
		let _b= sema.acquire().await.unwrap();
		let mut db = db.acquire().await?;
		let _result = sqlx::query!("INSERT INTO animelist (anidb_id, tvbid, series_title, def_tvdb_season, imdbid) VALUES (?,?,?,?,?);", entry.aid, entry.tvdbid, entry.series_title, entry.defaulttvdbseason, entry.imdbid)
			.execute(&mut db).await?;

		Ok(())
	}

	pub async fn fetch_new(&self) -> anyhow::Result<()> {
		let xml_data = reqwest::get("https://raw.githubusercontent.com/Anime-Lists/anime-lists/master/animetitles.xml").await?.bytes_stream();
		let x2 = xml_data.map(|x| {
			match x {
				Ok(b) => Ok(b),
				Err(e) => Err(Error::new(ErrorKind::Other, e.to_string()))
			}
		});
		let mut r = quick_xml::Reader::from_reader(StreamReader::new(x2));
		r.trim_text(true);

		let mut current_entry = AnimeEntry::default();

		let mut skip_current = false;

		let idx_count =  {
			let srch  = self.reader.searcher();
			srch.search(&tantivy::query::AllQuery {}, &collector::Count)?
			
		};

		
		let mut buf = Vec::new();

		println!("idx count {}", idx_count);


		loop {
            match r.read_event_into_async(&mut buf).await {
                Ok(event) => {
                    match event {
                        quick_xml::events::Event::Start(s) => {
                            let tag_name = String::from_utf8_lossy(s.name().0);
                             for attr in s.html_attributes().flatten() {
								skip_current = false;
								let (key,value) = (String::from_utf8_lossy(attr.key.0), String::from_utf8_lossy(&attr.value));
								
								if tag_name == "anime" && key == "aid" && !current_entry.title.is_empty() {
									if value.parse::<usize>().unwrap_or(0).lt(&idx_count) {
										// println!("already indexed {}", value);
										skip_current = true;
										break;
									}
									// save entry
									current_entry = AnimeEntry::default();
									current_entry.id = value.parse::<u64>().unwrap_or(0);
									break;
								} else if tag_name == "title" {
									break;
								}
                            }
                            
                        },
                        quick_xml::events::Event::Text(t) => {
                            if skip_current {
                                continue;
                            }
                            let text = t.unescape().unwrap();
							// new anime?
							current_entry.title = text.to_string();
							if current_entry.id != 0 {

								self.save_entry(&current_entry); // literally do not care if it fails
							}
                        },
                        quick_xml::events::Event::Eof => {
                            break;
                        },
						_ => {}
                    }
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                    return Err(anyhow::anyhow!("error parsing xml"));
                },
            }
        }

		Ok(())
	}
	

}

#[derive(Default,Debug,Clone,Serialize)]
pub struct AnimeEntry {
	pub id : u64,
	pub title: String
}
#[derive(Default,Debug,Clone,Serialize)]
pub struct SearchResult {
	pub entry : AnimeEntry,
	pub score : f32
}
#[derive(Default,Debug,Clone,Serialize)]
pub struct MasterEntry {
	aid: u32,
	tvdbid: u32,
	imdbid: Option<u32>,
	series_title: String,
	defaulttvdbseason: Option<u32>,
}