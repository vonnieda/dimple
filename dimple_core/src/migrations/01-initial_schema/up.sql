CREATE TABLE Metadata (
    key       TEXT PRIMARY KEY,
    value     TEXT
);

CREATE TABLE Blob (
    key        TEXT PRIMARY KEY,
    sha256     TEXT UNIQUE NOT NULL,
    length     U64 NOT NULL
);

CREATE TABLE Artist (
    key            TEXT PRIMARY KEY,
    name           TEXT,
    disambiguation TEXT,
    summary        TEXT,
    liked          BOOL NOT NULL DEFAULT false,
    country        TEXT
);

CREATE TABLE Track (
    key            TEXT PRIMARY KEY,
    artist         TEXT,
    album          TEXT,
    title          TEXT,
    liked          BOOL NOT NULL DEFAULT false,
    plays          INT NOT NULL DEFAULT 0,
    length_ms      INT,

    save           BOOL NOT NULL DEFAULT false,
    download       BOOL NOT NULL DEFAULT false,
    disambiguation TEXT,
    summary        TEXT,

    musicbrainz_id TEXT,
    wikidata_id    TEXT,
    spotify_id     TEXT,
    -- see comments on Track model
    -- discogs_id     TEXT,
    -- lastfm_id      TEXT,

    lyrics         TEXT,
    synced_lyrics  TEXT
);
CREATE INDEX Track_idx_1 ON Track (artist, album, title);

CREATE TABLE Genre (
    key            TEXT PRIMARY KEY,
    name           TEXT,
    disambiguation TEXT,
    summary        TEXT,
    liked          BOOL NOT NULL DEFAULT false
);

CREATE TABLE MediaFile (
    key       TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    sha256    TEXT NOT NULL,
    artist    TEXT,
    album     TEXT,
    title     TEXT
);

CREATE TABLE TrackSource (
    key            TEXT PRIMARY KEY,
    track_key      TEXT NOT NULL,
    blob_key       TEXT
    -- FOREIGN KEY (track_key) REFERENCES Track(key),
    -- FOREIGN KEY (blob_key) REFERENCES Blob(key)
);
CREATE INDEX TrackSource_idx_1 ON TrackSource (blob_key);
CREATE UNIQUE INDEX TrackSource_idx_2 ON TrackSource (track_key, blob_key);

CREATE TABLE Playlist (
    key       TEXT PRIMARY KEY,
    name      TEXT
);

CREATE TABLE PlaylistItem (
    key          TEXT PRIMARY KEY,
    -- ordinal      INT NOT NULL DEFAULT -1,
    playlist_key TEXT NOT NULL,
    track_key    TEXT NOT NULL,
    FOREIGN KEY (playlist_key) REFERENCES Playlist(key),
    FOREIGN KEY (track_key) REFERENCES Track(key)
);

CREATE TABLE ChangeLog (
    key       TEXT UNIQUE,
    actor     TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    model     TEXT NOT NULL,
    model_key TEXT NOT NULL,
    op        TEXT NOT NULL,
    field     TEXT,
    value     TEXT,
    PRIMARY KEY (actor, timestamp)
);
CREATE INDEX ChangeLog_idx_1 ON ChangeLog (model, model_key, field);

CREATE TABLE Event (
    key         TEXT PRIMARY KEY,
    timestamp   TEXT NOT NULL,
    event_type  TEXT NOT NULL,
    artist      TEXT,
    album       TEXT,
    title       TEXT,
    source_type TEXT NOT NULL,
    source      TEXT NOT NULL
);
CREATE INDEX Event_idx_1 ON Event (timestamp, event_type);
CREATE INDEX Event_idx_2 ON Event (timestamp);
CREATE INDEX Event_idx_3 ON Event (event_type);
CREATE UNIQUE INDEX Event_idx_4 ON Event (source_type, source);
