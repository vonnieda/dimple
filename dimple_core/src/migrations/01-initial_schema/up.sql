CREATE TABLE Artist (
    key TEXT PRIMARY KEY,
    name TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    country TEXT,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);
CREATE INDEX Artist_musicbrainz_id ON Artist (musicbrainz_id);
CREATE UNIQUE INDEX Artist_unique_name_disambiguation ON Artist (name, COALESCE(disambiguation, ''));

CREATE TABLE Release (
    key TEXT PRIMARY KEY,
    title TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    barcode TEXT,
    country TEXT,
    date TEXT,
    packaging TEXT,
    status TEXT,
    quality TEXT,
    release_group_type TEXT,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);
CREATE INDEX Release_title ON Release (title);
CREATE INDEX Release_musicbrainz_id ON Release (musicbrainz_id);

CREATE TABLE Track (
    key TEXT PRIMARY KEY,
    title TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    release_key TEXT,
    position INT,
    length_ms INT,
    lyrics TEXT,
    synchronized_lyrics TEXT,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT,

    media_track_count INT,
    media_position INT,
    media_title TEXT,
    media_format TEXT
);
CREATE INDEX Track_musicbrainz_id ON Track (musicbrainz_id);
CREATE INDEX Track_release_key ON Track (release_key);

CREATE TABLE Genre (
    key TEXT PRIMARY KEY,
    name TEXT UNIQUE NOT NULL,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);
CREATE INDEX Genre_musicbrainz_id ON Genre (musicbrainz_id);
CREATE UNIQUE INDEX Genre_unique_name_disambiguation ON Genre (name, COALESCE(disambiguation, ''));

CREATE TABLE Link (
    key TEXT PRIMARY KEY,
    name TEXT,
    url TEXT UNIQUE NOT NULL
);

CREATE TABLE Playlist (
    key TEXT PRIMARY KEY,
    name TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);
CREATE INDEX Playlist_musicbrainz_id ON Playlist (musicbrainz_id);

CREATE TABLE PlaylistItem (
    key TEXT PRIMARY KEY,
    playlist_key TEXT NOT NULL,
    ordinal TEXT NOT NULL,
    track_key TEXT NOT NULL,
    FOREIGN KEY (playlist_key) REFERENCES Playlist(key),
    FOREIGN KEY (track_key) REFERENCES Track(key)
);
CREATE INDEX PlaylistItem_playlist_key_ordinal ON PlaylistItem (playlist_key, ordinal);


CREATE TABLE LinkRef (
    model_key TEXT NOT NULL,
    link_key TEXT NOT NULL,
    FOREIGN KEY (link_key) REFERENCES Link(key)
);
CREATE UNIQUE INDEX LinkRef_unique_model_key_link_key ON LinkRef (model_key, link_key);

CREATE TABLE ArtistRef (
    model_key TEXT NOT NULL,
    artist_key TEXT NOT NULL,
    FOREIGN KEY (artist_key) REFERENCES Artist(key)
);
CREATE UNIQUE INDEX ArtistRef_unique_model_key_artist_key ON ArtistRef (model_key, artist_key);

CREATE TABLE GenreRef (
    model_key TEXT NOT NULL,
    genre_key TEXT NOT NULL,
    FOREIGN KEY (genre_key) REFERENCES Genre(key)
);
CREATE UNIQUE INDEX GenreRef_unique_model_key_genre_key ON GenreRef (model_key, genre_key);




CREATE TABLE MediaFile (
    key TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    sha256 TEXT NOT NULL,
    last_modified TEXT DEFAULT NULL,
    last_imported TEXT DEFAULT NULL
);

CREATE TABLE TrackSource (
    key TEXT PRIMARY KEY,
    track_key TEXT NOT NULL,
    blob_key TEXT,
    media_file_key TEXT,
    FOREIGN KEY (track_key) REFERENCES Track(key),
    FOREIGN KEY (blob_key) REFERENCES Blob(key),
    FOREIGN KEY (media_file_key) REFERENCES MediaFile(key)
);
CREATE INDEX TrackSource_idx_1 ON TrackSource (blob_key);
CREATE UNIQUE INDEX TrackSource_idx_2 ON TrackSource (track_key, blob_key);
CREATE INDEX TrackSource_idx_media_file_key ON TrackSource (media_file_key);



-- TODO Rename to Scrobble, I think.
CREATE TABLE Event (
    key TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    event_type TEXT NOT NULL,
    artist TEXT,
    album TEXT,
    title TEXT,
    source_type TEXT NOT NULL,
    source TEXT NOT NULL
);
CREATE INDEX Event_idx_1 ON Event (timestamp, event_type);
CREATE INDEX Event_idx_2 ON Event (timestamp);
CREATE INDEX Event_idx_3 ON Event (event_type);
CREATE UNIQUE INDEX Event_idx_4 ON Event (source_type, source);


CREATE TABLE Metadata (key TEXT PRIMARY KEY, value TEXT);

CREATE TABLE ChangeLog (
    key TEXT UNIQUE,
    actor TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    model TEXT NOT NULL,
    model_key TEXT NOT NULL,
    op TEXT NOT NULL,
    field TEXT,
    value TEXT,
    PRIMARY KEY (actor, timestamp)
);
CREATE INDEX ChangeLog_idx_1 ON ChangeLog (model, model_key, field);

CREATE TABLE Blob (
    key TEXT PRIMARY KEY,
    sha256 TEXT UNIQUE NOT NULL,
    length U64 NOT NULL
);


CREATE TABLE Dimage (
    key TEXT PRIMARY KEY,
    kind TEXT,
    width INT NOT NULL,
    height INT NOT NULL,
    png_thumbnail BLOB NOT NULL,
    png_data BLOB NOT NULL,
    sha256 UNIQUE NOT NULL
);

CREATE TABLE DimageRef (
    model_key TEXT NOT NULL,
    dimage_key TEXT NOT NULL,
    FOREIGN KEY (dimage_key) REFERENCES Dimage(key)
);
CREATE UNIQUE INDEX DimageRef_unique_model_key_dimage_key ON DimageRef (model_key, dimage_key);
