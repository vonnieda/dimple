CREATE TABLE Artist (
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
    title TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    release_id TEXT,
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
    media_format TEXT,
    FOREIGN KEY (release_id) REFERENCES Release(id)
);
CREATE INDEX Track_musicbrainz_id ON Track (musicbrainz_id);
CREATE INDEX Track_release_id ON Track (release_id);

CREATE TABLE Genre (
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
    name TEXT,
    url TEXT UNIQUE NOT NULL
);

CREATE TABLE Playlist (
    id TEXT PRIMARY KEY,
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
    id TEXT PRIMARY KEY,
    playlist_id TEXT NOT NULL,
    ordinal TEXT NOT NULL,
    track_id TEXT NOT NULL,
    FOREIGN KEY (playlist_id) REFERENCES Playlist(id),
    FOREIGN KEY (track_id) REFERENCES Track(id)
);
CREATE INDEX PlaylistItem_playlist_id_ordinal ON PlaylistItem (playlist_id, ordinal);

CREATE TABLE MediaFile (
    id TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    sha256 TEXT NOT NULL,
    last_modified TEXT DEFAULT NULL,
    last_imported TEXT DEFAULT NULL
);

CREATE TABLE TrackSource (
    id TEXT PRIMARY KEY,
    track_id TEXT NOT NULL,
    blob_id TEXT,
    media_file_id TEXT,
    FOREIGN KEY (track_id) REFERENCES Track(id),
    FOREIGN KEY (blob_id) REFERENCES Blob(id),
    FOREIGN KEY (media_file_id) REFERENCES MediaFile(id)
);
CREATE INDEX TrackSource_idx_1 ON TrackSource (blob_id);
CREATE UNIQUE INDEX TrackSource_idx_2 ON TrackSource (track_id, blob_id);
CREATE INDEX TrackSource_idx_media_file_id ON TrackSource (media_file_id);


CREATE TABLE Blob (
    id TEXT PRIMARY KEY,
    sha256 TEXT UNIQUE NOT NULL,
    length U32 NOT NULL
);


CREATE TABLE Dimage (
    id TEXT PRIMARY KEY,
    kind TEXT,
    width INT NOT NULL,
    height INT NOT NULL,
    png_thumbnail BLOB NOT NULL,
    png_data BLOB NOT NULL,
    sha256 UNIQUE NOT NULL
);

-- TODO Rename to Scrobble, I think.
CREATE TABLE Event (
    id TEXT PRIMARY KEY,
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

CREATE TABLE DimageRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    dimage_id TEXT NOT NULL,
    FOREIGN KEY (dimage_id) REFERENCES Dimage(id)
);
CREATE UNIQUE INDEX DimageRef_unique_model_id_dimage_id ON DimageRef (model_id, dimage_id);

CREATE TABLE LinkRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    link_id TEXT NOT NULL,
    FOREIGN KEY (link_id) REFERENCES Link(id)
);
CREATE UNIQUE INDEX LinkRef_unique_model_id_link_id ON LinkRef (model_id, link_id);

CREATE TABLE ArtistRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    artist_id TEXT NOT NULL,
    FOREIGN KEY (artist_id) REFERENCES Artist(id)
);
CREATE UNIQUE INDEX ArtistRef_unique_model_id_artist_id ON ArtistRef (model_id, artist_id);

CREATE TABLE GenreRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    genre_id TEXT NOT NULL,
    FOREIGN KEY (genre_id) REFERENCES Genre(id)
);
CREATE UNIQUE INDEX GenreRef_unique_model_id_genre_id ON GenreRef (model_id, genre_id);

