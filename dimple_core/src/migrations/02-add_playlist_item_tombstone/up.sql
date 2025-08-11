ALTER TABLE PlaylistItem ADD COLUMN deleted BOOLEAN NOT NULL DEFAULT FALSE;
CREATE INDEX PlaylistItem_deleted ON PlaylistItem (deleted);