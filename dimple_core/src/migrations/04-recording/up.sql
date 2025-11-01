CREATE TABLE Recording (
    id TEXT PRIMARY KEY,
    title TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    length_ms INT,
    lyrics TEXT,
    synchronized_lyrics TEXT,

    first_release_date TEXT,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);

