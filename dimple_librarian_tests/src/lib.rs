#[cfg(test)]
mod tests {
    use dimple_core::model::{Artist, KnownIds, Medium, Model, Recording, Release, ReleaseGroup, Track};
    use dimple_coverartarchive_plugin::CoverArtArchivePlugin;
    use dimple_fanart_tv_plugin::FanartTvPlugin;
    use dimple_librarian::librarian::Librarian;
    use dimple_musicbrainz_plugin::MusicBrainzPlugin;
    use dimple_theaudiodb_plugin::TheAudioDbPlugin;
    use dimple_wikidata_plugin::WikidataPlugin;

    // #[test]
    // fn shallow_models() {
    //     let librarian = Librarian::new_in_memory();
    //     librarian.add_plugin(Box::new(MusicBrainzPlugin::default()));
    //     librarian.add_plugin(Box::new(WikidataPlugin::default()));
    //     librarian.add_plugin(Box::new(FanartTvPlugin::default()));
    //     librarian.add_plugin(Box::new(TheAudioDbPlugin::default()));
    //     librarian.add_plugin(Box::new(CoverArtArchivePlugin::default()));

    //     // let mbid = "1cacfd62-b800-425e-b231-f90553b072e5".to_string();
    //     let mbid = "69158f97-4c07-4c4e-baf8-4e4ab1ed666e".to_string();
    //     let artist = librarian.get2(Artist::new_with_mbid(&mbid)).unwrap();
    //     let release_group = librarian.list2(ReleaseGroup::default(), Some(artist))
    //         .unwrap().next().unwrap();
    //     let release = librarian.list2(Release::default(), Some(release_group))
    //         .unwrap().next().unwrap();
    //     let medium = librarian.list2(Medium::default(), Some(release))
    //         .unwrap().next().unwrap();
    //     let track = librarian.list2(Track::default(), Some(medium))
    //         .unwrap().next().unwrap();
    //     // dbg!(&track);
    //     // let recording = librarian.list2(Recording::default(), Some(track))
    //     //     .unwrap().next().unwrap();
    //     // dbg!(&recording);
    // }

    #[test]
    fn shallow_models() {
        let librarian = Librarian::new_in_memory();
        librarian.add_plugin(Box::new(MusicBrainzPlugin::default()));
        librarian.add_plugin(Box::new(WikidataPlugin::default()));
        librarian.add_plugin(Box::new(FanartTvPlugin::default()));
        librarian.add_plugin(Box::new(TheAudioDbPlugin::default()));
        librarian.add_plugin(Box::new(CoverArtArchivePlugin::default()));

        // let mbid = "1cacfd62-b800-425e-b231-f90553b072e5".to_string();
        let mbid = "69158f97-4c07-4c4e-baf8-4e4ab1ed666e".to_string();
        let artist = librarian.get2(Artist::new_with_mbid(&mbid)).unwrap();
        let release_group = librarian.list2(ReleaseGroup::default(), Some(artist))
            .unwrap().next().unwrap();
        let release = librarian.list2(Release::default(), Some(release_group))
            .unwrap().next().unwrap();
        let medium = librarian.list2(Medium::default(), Some(release))
            .unwrap().next().unwrap();
        let track = librarian.list2(Track::default(), Some(medium))
            .unwrap().next().unwrap();
        let recording = librarian.list2(Recording::default(), Some(track))
            .unwrap().next().unwrap();
        assert!(recording.title == Some("Dayvan Cowboy".to_string()));
    }
}
