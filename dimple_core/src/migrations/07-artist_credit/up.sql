CREATE TABLE ArtistCredit (
    key          TEXT PRIMARY KEY,
    model_key    TEXT NOT NULL,
    artist_key   TEXT NOT NULL,
    FOREIGN KEY (artist_key) REFERENCES Artist(key)
);
CREATE UNIQUE INDEX ArtistCredit_idx_1 ON ArtistCredit (model_key, artist_key);
