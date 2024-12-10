PRAGMA journal_mode=WAL;

CREATE TABLE IF NOT EXISTS Metadata (
    key       TEXT PRIMARY KEY,
    value     TEXT
);

CREATE TABLE IF NOT EXISTS Blob (
    key        TEXT PRIMARY KEY,
    sha256     TEXT UNIQUE NOT NULL,
    length     U64 NOT NULL
);

CREATE TABLE IF NOT EXISTS Artist (
    key            TEXT PRIMARY KEY,
    name           TEXT,
    disambiguation TEXT,
    summary        TEXT,
    liked          BOOL NOT NULL DEFAULT false,
    country        TEXT
);

CREATE TABLE IF NOT EXISTS Track (
    key       TEXT PRIMARY KEY,
    artist    TEXT,
    album     TEXT,
    title     TEXT,
    liked     BOOL NOT NULL DEFAULT false,
    plays     INT NOT NULL DEFAULT 0,
    length_ms INT,

    lyrics    TEXT
);
CREATE INDEX IF NOT EXISTS Track_idx_1 ON Track (artist, album, title);

CREATE TABLE IF NOT EXISTS Genre (
    key            TEXT PRIMARY KEY,
    name           TEXT,
    disambiguation TEXT,
    summary        TEXT,
    liked          BOOL NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS MediaFile (
    key       TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    sha256    TEXT NOT NULL,
    artist    TEXT,
    album     TEXT,
    title     TEXT
);

CREATE TABLE IF NOT EXISTS TrackSource (
    key            TEXT PRIMARY KEY,
    track_key      TEXT NOT NULL,
    blob_key       TEXT
    -- FOREIGN KEY (track_key) REFERENCES Track(key),
    -- FOREIGN KEY (blob_key) REFERENCES Blob(key)
);
CREATE INDEX IF NOT EXISTS TrackSource_idx_1 ON TrackSource (blob_key);
CREATE UNIQUE INDEX IF NOT EXISTS TrackSource_idx_2 ON TrackSource (track_key, blob_key);

CREATE TABLE IF NOT EXISTS Playlist (
    key       TEXT PRIMARY KEY,
    name      TEXT
);

CREATE TABLE IF NOT EXISTS PlaylistItem (
    key          TEXT PRIMARY KEY,
    -- ordinal      INT NOT NULL DEFAULT -1,
    playlist_key TEXT NOT NULL,
    track_key    TEXT NOT NULL,
    FOREIGN KEY (playlist_key) REFERENCES Playlist(key),
    FOREIGN KEY (track_key) REFERENCES Track(key)
);

CREATE TABLE IF NOT EXISTS ChangeLog (
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
CREATE INDEX IF NOT EXISTS ChangeLog_idx_1 ON ChangeLog (model, model_key, field);

CREATE TABLE IF NOT EXISTS Event (
    key         TEXT PRIMARY KEY,
    timestamp   TEXT NOT NULL,
    event_type  TEXT NOT NULL,
    artist      TEXT,
    album       TEXT,
    title       TEXT,
    source_type TEXT NOT NULL,
    source      TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS Event_idx_1 ON Event (timestamp, event_type);
CREATE INDEX IF NOT EXISTS Event_idx_2 ON Event (timestamp);
CREATE INDEX IF NOT EXISTS Event_idx_3 ON Event (event_type);
CREATE UNIQUE INDEX IF NOT EXISTS Event_idx_4 ON Event (source_type, source);
