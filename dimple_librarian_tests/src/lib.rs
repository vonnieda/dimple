#[cfg(test)]
mod tests {
    use dimple_core::model::{Artist, KnownIds, ReleaseGroup};
    use dimple_coverartarchive_plugin::CoverArtArchivePlugin;
    use dimple_fanart_tv_plugin::FanartTvPlugin;
    use dimple_librarian::librarian::Librarian;
    use dimple_musicbrainz_plugin::MusicBrainzPlugin;
    use dimple_theaudiodb_plugin::TheAudioDbPlugin;
    use dimple_wikidata_plugin::WikidataPlugin;

    #[test]
    fn basics() {
        let librarian = Librarian::new("librarian_test");
        librarian.add_plugin(Box::new(MusicBrainzPlugin::default()));
        librarian.add_plugin(Box::new(WikidataPlugin::default()));
        librarian.add_plugin(Box::new(FanartTvPlugin::default()));
        librarian.add_plugin(Box::new(TheAudioDbPlugin::default()));
        librarian.add_plugin(Box::new(CoverArtArchivePlugin::default()));

        // let mbid = "1cacfd62-b800-425e-b231-f90553b072e5".to_string();
        let mbid = "69158f97-4c07-4c4e-baf8-4e4ab1ed666e".to_string();
        let artist = librarian.get2(
            Artist { 
                known_ids: KnownIds { 
                    musicbrainz_id: Some(mbid), 
                    ..Default::default() 
                },
             ..Default::default() 
        }).unwrap();
        dbg!(&artist);
        let release_groups = librarian.list2(ReleaseGroup::default(), Some(artist)).unwrap();
        for release_group in release_groups {
            dbg!(&release_group);
        }
    }
}
