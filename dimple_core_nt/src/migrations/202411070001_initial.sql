CREATE TABLE IF NOT EXISTS Metadata (
    key       TEXT PRIMARY KEY,
    value     TEXT
);

CREATE TABLE IF NOT EXISTS Artist (
    key       TEXT PRIMARY KEY,
    name      TEXT
);

CREATE TABLE IF NOT EXISTS Track (
    key       TEXT PRIMARY KEY,
    artist    TEXT,
    album     TEXT,
    title     TEXT,
    liked     BOOL NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS TrackSource (
    key       TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    artist    TEXT,
    album     TEXT,
    title     TEXT
    
    -- ,
    -- FOREIGN KEY (track_key) REFERENCES Track(key)
);

CREATE TABLE IF NOT EXISTS MediaFile (
    key       TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    artist    TEXT,
    album     TEXT,
    title     TEXT
);

CREATE TABLE IF NOT EXISTS Playlist (
    key       TEXT PRIMARY KEY,
    name      TEXT
);

CREATE TABLE IF NOT EXISTS PlaylistItem (
    key          TEXT PRIMARY KEY,
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

