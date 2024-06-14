#[cfg(test)]
mod tests {
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
    }
}
