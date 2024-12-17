ALTER TABLE MediaFile ADD COLUMN length_ms INT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN lyrics TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_artist_id TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_release_group_id TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_album_id TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_album_artist_id TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_track_id_id TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_recording_id TEXT DEFAULT NULL;
ALTER TABLE MediaFile ADD COLUMN musicbrainz_genre_id TEXT DEFAULT NULL;

