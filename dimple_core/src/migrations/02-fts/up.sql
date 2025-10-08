CREATE VIRTUAL TABLE ArtistFts USING fts5(
    name, 
    disambiguation, 
    summary, 
    country,
    discogs_id,
    lastfm_id,
    musicbrainz_id,
    spotify_id,
    wikidata_id,
    content='Artist', 
    content_rowid='rowid'
);

CREATE TRIGGER Artist_ai AFTER INSERT ON Artist BEGIN
  INSERT INTO ArtistFts(rowid, name, disambiguation, summary, country, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id) 
  VALUES (new.rowid, new.name, new.disambiguation, new.summary, new.country, new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id);
END;

CREATE TRIGGER Artist_ad AFTER DELETE ON Artist BEGIN
  INSERT INTO ArtistFts(ArtistFts, rowid, name, disambiguation, summary, country, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id) 
  VALUES('delete', old.rowid, old.name, old.disambiguation, old.summary, old.country, old.discogs_id, old.lastfm_id, old.musicbrainz_id, old.spotify_id, old.wikidata_id);
END;

CREATE TRIGGER Artist_au AFTER UPDATE ON Artist BEGIN
  INSERT INTO ArtistFts(ArtistFts, rowid, name, disambiguation, summary, country, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id) 
  VALUES('delete', old.rowid, old.name, old.disambiguation, old.summary, old.country, old.discogs_id, old.lastfm_id, old.musicbrainz_id, old.spotify_id, old.wikidata_id);
  INSERT INTO ArtistFts(rowid, name, disambiguation, summary, country, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id) 
  VALUES (new.rowid, new.name, new.disambiguation, new.summary, new.country, new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id);
END;

CREATE VIRTUAL TABLE ReleaseGroupFts USING fts5(
    title,
    disambiguation,
    summary,
    first_release_date,
    primary_type,
    discogs_id,
    lastfm_id,
    musicbrainz_id,
    spotify_id,
    wikidata_id,
    artist_names
);

CREATE TRIGGER ReleaseGroup_ai AFTER INSERT ON ReleaseGroup BEGIN
  INSERT OR REPLACE INTO ReleaseGroupFts(rowid, title, disambiguation, summary, first_release_date, primary_type, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id, artist_names)
  VALUES (new.rowid, new.title, new.disambiguation, new.summary, new.first_release_date, new.primary_type, new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id,
      (
      SELECT GROUP_CONCAT(name) 
      FROM Artist 
      JOIN ArtistRef
      ON ArtistRef.artist_id = Artist.id
      WHERE new.id = ArtistRef.model_id
    )
  );
END;

CREATE TRIGGER ReleaseGroup_ad AFTER DELETE ON ReleaseGroup BEGIN
  DELETE FROM ReleaseGroupFts WHERE rowid = old.rowid;
END;

CREATE TRIGGER ReleaseGroup_au AFTER UPDATE ON ReleaseGroup BEGIN
  INSERT OR REPLACE INTO ReleaseGroupFts(rowid, title, disambiguation, summary, first_release_date, primary_type, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id, artist_names)
  VALUES (new.rowid, new.title, new.disambiguation, new.summary, new.first_release_date, new.primary_type, new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id,
      (
      SELECT GROUP_CONCAT(name) 
      FROM Artist 
      JOIN ArtistRef
      ON ArtistRef.artist_id = Artist.id
      WHERE new.id = ArtistRef.model_id
    )
  );
END;

CREATE VIRTUAL TABLE ReleaseFts USING fts5(
    title,
    disambiguation,
    summary,
    barcode,
    country,
    date,
    packaging,
    status,
    quality,
    discogs_id,
    lastfm_id,
    musicbrainz_id,
    spotify_id,
    wikidata_id,
    artist_names
);

CREATE TRIGGER Release_ai AFTER INSERT ON Release BEGIN
  INSERT OR REPLACE INTO ReleaseFts(
    rowid, title, disambiguation, summary, barcode, country, date, packaging, 
    status, quality, discogs_id, lastfm_id, musicbrainz_id, spotify_id, 
    wikidata_id, artist_names
  )
  VALUES (
    new.rowid, new.title, new.disambiguation, new.summary, new.barcode, 
    new.country, new.date, new.packaging, new.status, new.quality, 
    new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, 
    new.wikidata_id, 
    (
      SELECT GROUP_CONCAT(name) 
      FROM Artist 
      JOIN ArtistRef
      ON ArtistRef.artist_id = Artist.id
      WHERE new.id = ArtistRef.model_id
    )
  );
END;

CREATE TRIGGER Release_ad AFTER DELETE ON Release BEGIN
  DELETE FROM ReleaseFts WHERE rowid = old.rowid;
END;

CREATE TRIGGER Release_au AFTER UPDATE ON Release BEGIN
  INSERT OR REPLACE INTO ReleaseFts(
    rowid, title, disambiguation, summary, barcode, country, date, packaging, 
    status, quality, discogs_id, lastfm_id, musicbrainz_id, spotify_id, 
    wikidata_id, artist_names
  )
  VALUES (
    new.rowid, new.title, new.disambiguation, new.summary, new.barcode, 
    new.country, new.date, new.packaging, new.status, new.quality, 
    new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, 
    new.wikidata_id, 
    (
      SELECT GROUP_CONCAT(name) 
      FROM Artist 
      JOIN ArtistRef
      ON ArtistRef.artist_id = Artist.id
      WHERE new.id = ArtistRef.model_id
    )
  );
END;

CREATE VIRTUAL TABLE TrackFts USING fts5(
    title,
    disambiguation,
    summary,
    lyrics,
    synchronized_lyrics,
    media_title,
    media_format,
    discogs_id,
    lastfm_id,
    musicbrainz_id,
    spotify_id,
    wikidata_id,
    artist_names
);

CREATE TRIGGER Track_ai AFTER INSERT ON Track BEGIN
  INSERT OR REPLACE INTO TrackFts(rowid, title, disambiguation, summary, lyrics, 
  synchronized_lyrics, media_title, media_format, discogs_id, lastfm_id, 
  musicbrainz_id, spotify_id, wikidata_id, artist_names)
  VALUES (new.rowid, new.title, new.disambiguation, new.summary, new.lyrics, 
  new.synchronized_lyrics, new.media_title, new.media_format, new.discogs_id, 
  new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id,
    (
      SELECT GROUP_CONCAT(name) 
      FROM Artist 
      JOIN ArtistRef
      ON ArtistRef.artist_id = Artist.id
      WHERE new.id = ArtistRef.model_id
    )
  );
END;

CREATE TRIGGER Track_ad AFTER DELETE ON Track BEGIN
  DELETE FROM TrackFts WHERE rowid = old.rowid;
END;

CREATE TRIGGER Track_au AFTER UPDATE ON Track BEGIN
  INSERT OR REPLACE INTO TrackFts(rowid, title, disambiguation, summary, lyrics, 
  synchronized_lyrics, media_title, media_format, discogs_id, lastfm_id, 
  musicbrainz_id, spotify_id, wikidata_id, artist_names)
  VALUES (new.rowid, new.title, new.disambiguation, new.summary, new.lyrics, 
  new.synchronized_lyrics, new.media_title, new.media_format, new.discogs_id, 
  new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id,
    (
      SELECT GROUP_CONCAT(name) 
      FROM Artist 
      JOIN ArtistRef
      ON ArtistRef.artist_id = Artist.id
      WHERE new.id = ArtistRef.model_id
    )
  );
END;

CREATE VIRTUAL TABLE GenreFts USING fts5(
    name,
    disambiguation,
    summary,
    discogs_id,
    lastfm_id,
    musicbrainz_id,
    spotify_id,
    wikidata_id,
    content='Genre',
    content_rowid='rowid'
);

CREATE TRIGGER Genre_ai AFTER INSERT ON Genre BEGIN
  INSERT INTO GenreFts(rowid, name, disambiguation, summary, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id)
  VALUES (new.rowid, new.name, new.disambiguation, new.summary, new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id);
END;

CREATE TRIGGER Genre_ad AFTER DELETE ON Genre BEGIN
  INSERT INTO GenreFts(GenreFts, rowid, name, disambiguation, summary, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id)
  VALUES('delete', old.rowid, old.name, old.disambiguation, old.summary, old.discogs_id, old.lastfm_id, old.musicbrainz_id, old.spotify_id, old.wikidata_id);
END;

CREATE TRIGGER Genre_au AFTER UPDATE ON Genre BEGIN
  INSERT INTO GenreFts(GenreFts, rowid, name, disambiguation, summary, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id)
  VALUES('delete', old.rowid, old.name, old.disambiguation, old.summary, old.discogs_id, old.lastfm_id, old.musicbrainz_id, old.spotify_id, old.wikidata_id);
  INSERT INTO GenreFts(rowid, name, disambiguation, summary, discogs_id, lastfm_id, musicbrainz_id, spotify_id, wikidata_id)
  VALUES (new.rowid, new.name, new.disambiguation, new.summary, new.discogs_id, new.lastfm_id, new.musicbrainz_id, new.spotify_id, new.wikidata_id);
END;

