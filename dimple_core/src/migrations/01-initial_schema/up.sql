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
    release_group_id TEXT,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);
CREATE INDEX Release_title ON Release (title);
CREATE INDEX Release_musicbrainz_id ON Release (musicbrainz_id);

CREATE TABLE ReleaseGroup (
    id TEXT PRIMARY KEY,
    title TEXT,
    disambiguation TEXT,
    summary TEXT,
    save BOOL NOT NULL DEFAULT false,
    download BOOL NOT NULL DEFAULT false,

    first_release_date TEXT,
    primary_type TEXT,

    discogs_id TEXT,
    lastfm_id TEXT,
    musicbrainz_id TEXT,
    spotify_id TEXT,
    wikidata_id TEXT
);
CREATE INDEX ReleaseGroup_title ON ReleaseGroup (title);
CREATE INDEX ReleaseGroup_musicbrainz_id ON ReleaseGroup (musicbrainz_id);

CREATE TABLE ReleaseGroupSecondaryTypeRef (
    id TEXT PRIMARY KEY,
    release_group_id TEXT NOT NULL,
    secondary_type TEXT NOT NULL
);

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
    media_format TEXT
);
CREATE INDEX Track_musicbrainz_id ON Track (musicbrainz_id);
CREATE INDEX Track_release_id ON Track (release_id);

CREATE TABLE Genre (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
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

CREATE TABLE Link (
    id TEXT PRIMARY KEY,
    name TEXT,
    url TEXT NOT NULL
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
    deleted BOOLEAN NOT NULL DEFAULT FALSE
);
CREATE INDEX PlaylistItem_playlist_id_ordinal ON PlaylistItem (playlist_id, ordinal);
CREATE INDEX PlaylistItem_deleted ON PlaylistItem (deleted);

CREATE TABLE Dimage (
    id TEXT PRIMARY KEY,
    kind TEXT,
    width INT NOT NULL,
    height INT NOT NULL,
    png_thumbnail BLOB NOT NULL,
    png_data BLOB NOT NULL,
    sha256 NOT NULL
);

CREATE TABLE Scrobble (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    scrobble_type TEXT NOT NULL,
    artist TEXT,
    album TEXT,
    title TEXT,
    source_type TEXT NOT NULL,
    source TEXT NOT NULL
);
CREATE INDEX Scrobble_idx_1 ON Scrobble (timestamp, scrobble_type);
CREATE INDEX Scrobble_idx_2 ON Scrobble (timestamp);
CREATE INDEX Scrobble_idx_3 ON Scrobble (scrobble_type);
CREATE INDEX Scrobble_idx_4 ON Scrobble (source_type, source);

CREATE TABLE MediaFile (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    last_modified TEXT DEFAULT NULL,
    last_imported TEXT DEFAULT NULL,
    content BLOB -- TODO this isn't where I want this, but just wanna see it working
);

CREATE TABLE TrackSource (
    id TEXT PRIMARY KEY,
    track_id TEXT NOT NULL,
    media_file_id TEXT
);
CREATE INDEX TrackSource_idx_media_file_id ON TrackSource (media_file_id);

CREATE TABLE DimageRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    dimage_id TEXT NOT NULL
);
CREATE INDEX DimageRef_model_id_dimage_id ON DimageRef (model_id, dimage_id);

CREATE TABLE LinkRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    link_id TEXT NOT NULL
);
CREATE INDEX LinkRef_model_id_link_id ON LinkRef (model_id, link_id);

CREATE TABLE ArtistRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    artist_id TEXT NOT NULL
);
CREATE INDEX ArtistRef_model_id_artist_id ON ArtistRef (model_id, artist_id);

CREATE TABLE GenreRef (
    id TEXT PRIMARY KEY,
    model_id TEXT NOT NULL,
    genre_id TEXT NOT NULL
);
CREATE INDEX GenreRef_model_id_genre_id ON GenreRef (model_id, genre_id);

