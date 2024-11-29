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
    length_ms INT
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

-- Note because I keep forgetting it myself: There can be multiple TrackSources
-- with the same media_file_key for various reasons. For example, a greatest
-- hits may include the exact recording from the original hit and would thus
-- reference the same piece of media.
CREATE TABLE IF NOT EXISTS TrackSource (
    key            TEXT PRIMARY KEY,
    track_key      TEXT NOT NULL,
    blob_key       TEXT
    -- TODO blobs, urls, etc.
    -- TODO probably unique across that plus track_key
    -- FOREIGN KEY (track_key) REFERENCES Track(key) -- TODO breaks a test cause no tracks exist
);
CREATE INDEX IF NOT EXISTS TrackSource_idx_1 ON TrackSource (blob_key);

CREATE TABLE IF NOT EXISTS Playlist (
    key       TEXT PRIMARY KEY,
    name      TEXT
);

CREATE TABLE IF NOT EXISTS PlaylistItem (
    key          TEXT PRIMARY KEY,
    -- TODO ordinal, probably
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

