ALTER TABLE MediaFile ADD COLUMN genre TEXT DEFAULT NULL;
ALTER TABLE MediaFile RENAME COLUMN musicbrainz_track_id_id TO musicbrainz_track_id;
