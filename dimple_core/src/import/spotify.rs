use std::fs::{self, File};

use crate::{library::Library, model::Event};

use chrono::DateTime;
use walkdir::{DirEntry, WalkDir};

use serde::{Deserialize, Serialize};

pub fn import(library: &Library, path: &str) {
    let json_files = WalkDir::new(path).into_iter()
        .filter(|dir_entry| dir_entry.is_ok())
        .map(|dir_entry| dir_entry.unwrap())
        .filter(|dir_entry| dir_entry.file_type().is_file() 
            && dir_entry.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("json")))
        .collect::<Vec<_>>();
    // dbg!(&json_files);
    for json_file in json_files.iter() {
        let filename = json_file.file_name().to_str().unwrap();
        if filename.contains("Streaming_History_Audio") {
            import_streaming_history_audio(library, json_file);
        }
    }
}

fn import_streaming_history_audio(library: &Library, json_file: &DirEntry) {
    log::info!("Importing Spotify Streaming_History_Audio file {:?}", json_file.path().as_os_str());
    let json = fs::read_to_string(json_file.path()).unwrap();
    let entries: Vec<StreamingHistoryAudioEntry> = serde_json::from_str(&json).unwrap();
    
    for (i, entry) in entries.iter().enumerate() {
        if !(entry.ts.is_some() 
            && entry.master_metadata_album_artist_name.is_some() 
            && entry.master_metadata_track_name.is_some()) {
                log::warn!("Invalid entry #{}. Missing ts, artist, or title.", i);
                continue
        }         
        // There is a unique index on (source_type, source) so if we're
        // re-importing the same data we'll just update the existing row.
        // TODO no longer true since removing upsert, will blow up
        library.save(&Event {
            timestamp: DateTime::parse_from_rfc3339(&entry.ts.clone().unwrap()).unwrap().into(),
            event_type: match entry.skipped {
                Some(true) => "track_skipped",
                _ => "track_played",
            }.to_string(),
            artist: entry.master_metadata_album_artist_name.clone(),
            album: entry.master_metadata_album_album_name.clone(),
            title: entry.master_metadata_track_name.clone(),
            source_type: "spotify::StreamingHistoryAudioEntry".to_string(),
            source: serde_json::to_string(entry).unwrap(),
            ..Default::default()
        });
    }
}

#[derive(Debug, Deserialize, Clone, Serialize)]
struct StreamingHistoryAudioEntry {
    pub ts: Option<String>,
    pub master_metadata_track_name: Option<String>,
    pub master_metadata_album_artist_name: Option<String>,
    pub master_metadata_album_album_name: Option<String>,
    pub spotify_track_uri: Option<String>,
    pub ms_played: Option<u64>,
    pub skipped: Option<bool>,
    pub reason_start: Option<String>,
    pub reason_end: Option<String>,
}

// {
//     "ts": "2011-07-15T18:55:24Z",
//     "username": "jvonnieda",
//     "platform": "OS X 10.6.8 [x86 4]",
//     "ms_played": 71471,
//     "conn_country": "US",
//     "ip_addr_decrypted": "98.237.245.108",
//     "user_agent_decrypted": null,
//     "master_metadata_track_name": "Mandala",
//     "master_metadata_album_artist_name": "Morcheeba",
//     "master_metadata_album_album_name": "Blood Like Lemonade",
//     "spotify_track_uri": "spotify:track:3OO3GT7Batv8P0dnecrDdf",
//     "episode_name": null,
//     "episode_show_name": null,
//     "spotify_episode_uri": null,
//     "reason_start": "trackdone",
//     "reason_end": "popup",
//     "shuffle": false,
//     "skipped": true,
//     "offline": false,
//     "offline_timestamp": 0,
//     "incognito_mode": null
//   },

#[cfg(test)]
mod tests {
    use crate::{import, library::Library, model::Event};

    #[test]
    fn it_works() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        assert!(library.list::<Event>().len() == 0);
        import::spotify::import(&library, "tests/data/spotify_history");
        assert!(library.list::<Event>().len() > 0);
    }
}

