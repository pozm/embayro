-- Add migration script here
CREATE TABLE IF NOT EXISTS animelist (
	anidb_id INTEGER PRIMARY KEY,
	series_title TEXT,
	tvbid INTEGER,
	def_tvdb_season INTEGER,
	ep_offset INTEGER,
	tmbid INTEGER,
	imdbid INTEGER

);
CREATE INDEX i_tvbid ON animelist (tvbid);