use std::{io::{Cursor}, sync::Arc};

use config::Config;
use crossbeam::channel::{Receiver, unbounded};
use crypto::sha2::Sha256;
use crypto::digest::Digest;
use image::DynamicImage;
use log::{debug};
use rodio::{Decoder, Sink};
use sunk::{Client, search::SearchPage, ListType, Album, Media, song::Song, Streamable, Artist};
use url::Url;


use super::{Library, Release, Image, Genre, Track, image_cache::ImageCache};

use std::iter::Iterator;

use rayon::prelude::*;

use std::thread;

pub struct NavidromeLibrary {
    site: String,
    username: String,
    password: String,
    image_cache: ImageCache,
}

/// Album { 
///     id: "0bdf4f9411c8f162927d26a6b2a11bf4", 
///     name: "Movements", 
///     artist: Some("Booka Shade"), 
///     artist_id: Some("3b4cbe54226a4af13f7af3a474c41042"), 
///     cover_id: Some("al-0bdf4f9411c8f162927d26a6b2a11bf4_63fbbde8"), 
///     duration: 3600, 
///     year: Some(2006), 
///     genre: Some("Electro"), 
///     song_count: 12, 
///     songs: [] 
/// }
/// 
/// Merging Release { 
///     url: "navidrome:///jason/release/0bdf4f9411c8f162927d26a6b2a11bf4", 
///     title: "Movements", 
///     artists: [
///         Artist { 
///             url: "navidrome:///jason/artist/Booka Shade", 
///             name: "Booka Shade", 
///             art: [] 
///         }
///     ], 
///     art: [
///         Image { 
///             url: "navidrome:///jason/image/al-0bdf4f9411c8f162927d26a6b2a11bf4_63fbbde8" 
///         }
///     ], 
///     genres: [
///         Genre { 
///             url: "navidrome:///jason/genre/Electro", 
///             name: "Electro", 
///             art: [] 
///         }
///     ], 
///     tracks: [] 
/// }    
/// 

impl Library for NavidromeLibrary {
    fn name(&self) -> String {
        return self.base_url();
    }

    fn releases(&self) -> Receiver<Release> {
        let client = Arc::new(Box::new(self.new_client().unwrap()));
        let (sender, receiver) = unbounded::<Release>();
        let base_url = self.base_url();

        thread::spawn(move || {
            let sender = sender.clone();

            // Get all the artists
            sunk::Artist::list(&client, 0).unwrap()
                .par_iter()

                // .map(|raw_artist| super::Artist::from(raw_artist))

                // Get all albums for each artist. Does not include songs.
                .flat_map(|artist| {
                    let albums = artist.albums(&client).unwrap();
                    albums.into_iter().map(move |album| (artist.clone(), album)).collect::<Vec<_>>()
                })

                // Get full album, including songs, for each album.
                .map(|(artist, shallow_album)| {
                    (artist, Album::get(&client, &shallow_album.id).unwrap())
                })

                // Convert each album to a release
                .map(|(artist, album)| {
                    Self::sunk_artist_album_to_dimple_release(&base_url, 
                        &artist, 
                        &album)
                })

                // And send back to the caller.
                .for_each(|release| {
                    sender.clone().send(release).unwrap();
                });
        });

        return receiver;            
    }

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        let (_object_type, id) = self.un_url(&image.url);

        // check cache first
        if let Some(image) = self.image_cache.get_original(&id) {
            return Ok(image);
        }

        // not found, download it
        let album = sunk::Album {
            cover_id: Some(id.to_string()),
            ..Default::default()
        };
        debug!("Downloading {}", id);
        let client = self.new_client()?;
        let bytes = album.cover_art(&client, 0).map_err(|e| e.to_string())?;
        let dynamic_image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;

        // cache it
        self.image_cache.insert(&id, &dynamic_image);

        // return it
        return Ok(dynamic_image);
    }

    fn stream(&self, track: &Track, sink: &Sink) -> Result<(), String> {
        let (_object_type, id) = self.un_url(&track.url);
        let client = self.new_client()?;
        let song = Song::get(&client, &id).unwrap();
        let stream = song.stream(&client).unwrap();
        let source = Decoder::new(Cursor::new(stream)).unwrap();
        sink.append(source);
        Ok(())
    }
}

impl NavidromeLibrary {
    pub fn new(site: &str, username: &str, password: &str) -> Self {
        let db = sled::open("data/navidrome").unwrap();
        Self {
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
            image_cache: ImageCache::new(db.open_tree("image_cache").unwrap()),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self::new(config.get_string("navidrome.site").unwrap().as_str(),
            config.get_string("navidrome.username").unwrap().as_str(),
            config.get_string("navidrome.password").unwrap().as_str())
    }

    fn base_url(&self) -> String {
        format!("navidrome:///{}", self.username)
    }

    fn url(base_url: &str, object_type: &str, id: &str) -> String {
        format!("{}/{}/{}", base_url, object_type, id)
    }

    fn un_url(&self, url: &str) -> (String, String) {
        let url = Url::parse(url).unwrap();
        let mut path_segments = url.path_segments().unwrap();
        let _username = path_segments.next().unwrap().to_string();
        let object_type = path_segments.next().unwrap().to_string();
        let id = path_segments.next().unwrap().to_string();
        return (object_type, id);
    }

    fn new_client(&self) -> Result<Client, String> {
        sunk::Client::new(
            self.site.as_str(),
            self.username.as_str(),
            self.password.as_str(),
        ).map_err(|e| e.to_string())
    }

    fn sunk_artist_album_to_dimple_release(base_url: &str, 
        artist: &sunk::Artist,
        album: &sunk::Album) -> crate::music_library::Release {
            let url = Self::url(base_url, "album", &album.id); 
            let title = album.name.clone();
            let artists = vec![
                Self::sunk_artist_to_dimple_artist(base_url, &artist)
            ]; 
            let tracks = album.songs.iter().map(|song| {
                Self::sunk_song_to_dimple_track(base_url, &song)
            }).collect();
            let art = album.cover_id().map_or(
                vec![], 
                |cover_id| vec![Self::sunk_image_to_dimple_image(base_url, cover_id)]
            );
            let genres = album.genre.as_ref().map_or(
                vec![], 
                |genre_name| vec![Self::sunk_genre_to_dimple_genre(base_url, &genre_name)]
            );
            Release {
                url,
                title,
                artists,
                tracks,
                art,
                genres,
            }
    }

    fn sunk_artist_to_dimple_artist(base_url: &str, artist: &sunk::Artist) -> crate::music_library::Artist {
        return crate::music_library::Artist {
            url: Self::url(base_url, "artist", &artist.id),
            name: artist.name.clone(),
            art: vec![],
        };
    }

    fn sunk_image_to_dimple_image(base_url: &str, image_id: &str) -> crate::music_library::Image {
        crate::music_library::Image {
            url: Self::url(base_url, "image", image_id)
        }
    }

    fn sunk_genre_to_dimple_genre(base_url: &str, genre_name: &str) -> crate::music_library::Genre {
        crate::music_library::Genre {
            url: Self::url(base_url, "genre", &Self::string_to_id(&genre_name)),
            name: genre_name.to_string(),
            art: vec![],
        }
    }

    fn sunk_song_to_dimple_track(base_url: &str, song: &sunk::song::Song) -> crate::music_library::Track {
        Track {
            url: Self::url(base_url, "track", &song.id),
            title: song.title.clone(),
            ..Default::default() 
        }
    }

    fn string_to_id(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(input);
        return hasher.result_str();
    }
}

// http://your-server/rest/getArtists Since 1.8.0 
// <subsonic-response status="ok" version="1.10.1">
// <artists ignoredArticles="The El La Los Las Le Les">
// <index name="A">
// <artist id="5449" name="A-Ha" coverArt="ar-5449" albumCount="4"/>
// <artist id="5421" name="ABBA" coverArt="ar-5421" albumCount="6"/>
// <artist id="5432" name="AC/DC" coverArt="ar-5432" albumCount="15"/>
// <artist id="6633" name="Aaron Neville" coverArt="ar-6633" albumCount="1"/>
// </index>
// <index name="B">
// <artist id="5950" name="Bob Marley" coverArt="ar-5950" albumCount="8"/>
// <artist id="5957" name="Bruce Dickinson" coverArt="ar-5957" albumCount="2"/>
// </index>
// </artists>
// </subsonic-response>

// http://your-server/rest/getAlbumList2
// <subsonic-response status="ok" version="1.8.0">
// <albumList2>
// <album id="1768" name="Duets" coverArt="al-1768" songCount="2" created="2002-11-09T15:44:40" duration="514" artist="Nik Kershaw" artistId="829"/>
// <album id="2277" name="Hot" coverArt="al-2277" songCount="4" created="2004-11-28T00:06:52" duration="1110" artist="Melanie B" artistId="1242"/>
// <album id="4201" name="Bande A Part" coverArt="al-4201" songCount="14" created="2007-10-29T19:25:05" duration="3061" artist="Nouvelle Vague" artistId="2060"/>
// <album id="2910" name="Soundtrack From Twin Peaks" coverArt="al-2910" songCount="6" created="2002-11-17T09:58:42" duration="1802" artist="Angelo Badalamenti" artistId="1515"/>
// <album id="3109" name="Wild One" coverArt="al-3109" songCount="38" created="2001-04-17T00:20:08" duration="9282" artist="Thin Lizzy" artistId="661"/>
// <album id="1151" name="Perleporten" coverArt="al-1151" songCount="2" created="2002-11-16T22:24:22" duration="494" artist="Magnus Grønneberg" artistId="747"/>
// <album id="2204" name="Wholesale Meats And Fish" coverArt="al-2204" songCount="24" created="2004-11-27T23:44:31" duration="5362" artist="Letters To Cleo" artistId="1216"/>
// <album id="114" name="Sounds of the Seventies: AM Nuggets" coverArt="al-114" songCount="2" created="2004-03-09T07:32:46" duration="420" artist="Rubettes" artistId="97"/>
// <album id="279" name="Waiting for the Day" coverArt="al-279" songCount="2" created="2004-11-27T17:49:19" duration="448" artist="Bachelor Girl" artistId="231"/>
// <album id="4414" name="For Sale" songCount="14" created="2007-10-30T00:11:58" duration="2046" artist="The Beatles" artistId="509"/>
// </albumList2>
// </subsonic-response>


// http://your-server/rest/getAlbum 
// <subsonic-response status="ok" version="1.8.0">
// <album id="11053" name="High Voltage" coverArt="al-11053" songCount="8" created="2004-11-27T20:23:32" duration="2414" artist="AC/DC" artistId="5432">
// <song id="71463" parent="71381" title="The Jack" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-08T23:36:11" duration="352" bitRate="128" size="5624132" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - The Jack.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71464" parent="71381" title="Tnt" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-08T23:36:11" duration="215" bitRate="128" size="3433798" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - TNT.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71458" parent="71381" title="It's A Long Way To The Top" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-27T20:23:32" duration="315" bitRate="128" year="1976" genre="Rock" size="5037357" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - It's a long way to the top if you wanna rock 'n 'roll.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71461" parent="71381" title="Rock 'n' Roll Singer." album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-27T20:23:33" duration="303" bitRate="128" track="2" year="1976" genre="Rock" size="4861680" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - Rock N Roll Singer.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71460" parent="71381" title="Live Wire" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-27T20:23:33" duration="349" bitRate="128" track="4" year="1976" genre="Rock" size="5600206" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - Live Wire.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71456" parent="71381" title="Can I sit next to you girl" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-27T20:23:32" duration="251" bitRate="128" track="6" year="1976" genre="Rock" size="4028276" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - Can I Sit Next To You Girl.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71459" parent="71381" title="Little Lover" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-27T20:23:33" duration="339" bitRate="128" track="7" year="1976" genre="Rock" size="5435119" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - Little Lover.mp3" albumId="11053" artistId="5432" type="music"/>
// <song id="71462" parent="71381" title="She's Got Balls" album="High Voltage" artist="AC/DC" isDir="false" coverArt="71381" created="2004-11-27T20:23:34" duration="290" bitRate="128" track="8" year="1976" genre="Rock" size="4651866" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/High voltage/ACDC - Shes Got Balls.mp3" albumId="11053" artistId="5432" type="music"/>
// </album>
// </subsonic-response>

// http://your-server/rest/getArtist
// <subsonic-response status="ok" version="1.8.0">
// <artist id="5432" name="AC/DC" coverArt="ar-5432" albumCount="15">
// <album id="11047" name="Back In Black" coverArt="al-11047" songCount="10" created="2004-11-08T23:33:11" duration="2534" artist="AC/DC" artistId="5432"/>
// ..
// <album id="11061" name="Who Made Who" coverArt="al-11061" songCount="9" created="2004-11-08T23:43:18" duration="2291" artist="AC/DC" artistId="5432"/>
// </artist>
// </subsonic-response>


// http://your-server/rest/getSong
// <subsonic-response status="ok" version="1.8.0">
// <song id="48228" parent="48203" title="You Shook Me All Night Long" album="Back In Black" artist="AC/DC" isDir="false" coverArt="48203" created="2004-11-08T23:33:11" duration="210" bitRate="112" size="2945619" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/Back in black/ACDC - You Shook Me All Night Long.mp3"/>
// </subsonic-response>

// http://your-server/rest/getGenres
// <subsonic-response status="ok" version="1.10.2">
// <genres>
// <genre songCount="28" albumCount="6">Electronic</genre>
// <genre songCount="6" albumCount="2">Hard Rock</genre>
// <genre songCount="8" albumCount="2">R&B</genre>
// <genre songCount="22" albumCount="2">Blues</genre>
// <genre songCount="2" albumCount="2">Podcast</genre>
// <genre songCount="11" albumCount="1">Brit Pop</genre>
// <genre songCount="14" albumCount="1">Live</genre>
// </genres>
// </subsonic-response>

// {
//     "ignoredArticles": "The El La Los Las Le Les Os As O A",
//     "index": [
//         {
//             "artist": [
//                 {
//                     "albumCount": 1,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLTBlOWFiNjBhNmY3MDFiM2E3MjdlZjhkNzc0YmQwMGUxXzAiLCJpc3MiOiJORCJ9.RidGv5wzDOaia0Hnp4tHvLvzbydc7Kms1mf02fUU8ic?size=600",
//                     "coverArt": "ar-0e9ab60a6f701b3a727ef8d774bd00e1_0",
//                     "id": "0e9ab60a6f701b3a727ef8d774bd00e1",
//                     "name": "1000 Clowns"
//                 },
//                 {
//                     "albumCount": 1,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLWZkNDJmZWFlZjM1ZDUwNzU3NDA1OWEwYjRkMWZhZTU0XzAiLCJpc3MiOiJORCJ9.v9Y5wxaWLpirnliBQAdit6X6yxHB9XTz3DWq4AwPCw4?size=600",
//                     "coverArt": "ar-fd42feaef35d507574059a0b4d1fae54_0",
//                     "id": "fd42feaef35d507574059a0b4d1fae54",
//                     "name": "30 Seconds to Mars"
//                 },
//                 {
//                     "albumCount": 7,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLTlkZmNkNWU1NThkZmEwNGFhZjM3ZjEzN2ExZDlkM2U1XzAiLCJpc3MiOiJORCJ9.9Ay-9D9y8X5xEQZnOuHz3ZczGGVmGhxX20GKaI8iXzI?size=600",
//                     "coverArt": "ar-9dfcd5e558dfa04aaf37f137a1d9d3e5_0",
//                     "id": "9dfcd5e558dfa04aaf37f137a1d9d3e5",
//                     "name": "311"
//                 },
//                 {
//                     "albumCount": 4,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLTczMzJiMWFlMTNmZjhjYmUzYTNlOTFkOGQwNGMzODk1XzAiLCJpc3MiOiJORCJ9.YlHZwUOW4ZVSXG2CKlgaqlEGpgHVMsiToVrg6hN4LIs?size=600",
//                     "coverArt": "ar-7332b1ae13ff8cbe3a3e91d8d04c3895_0",
//                     "id": "7332b1ae13ff8cbe3a3e91d8d04c3895",
//                     "name": "40 Watt Sun"
//                 },
//                 {
//                     "albumCount": 1,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLTNjOGRiZTBmMWI0M2YwNzU0ZGEwMTcxYjJiZWE1ZjUxXzAiLCJpc3MiOiJORCJ9.W84StXUgt1jhu0um1HzsTe5rfuGnTe9rXPq7pDIVG8o?size=600",
//                     "coverArt": "ar-3c8dbe0f1b43f0754da0171b2bea5f51_0",
//                     "id": "3c8dbe0f1b43f0754da0171b2bea5f51",
//                     "name": "Æther Realm"
//                 },
//                 {
//                     "albumCount": 1,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLTFmN2VlMTI0NDc4M2UwZjYwMTBhZWI5ZjM1YzM4NGE3XzAiLCJpc3MiOiJORCJ9.fxUiWrgXc427KGfmbxxwpmlpbgxdm4g_yd2T-R8o_cw?size=600",
//                     "coverArt": "ar-1f7ee1244783e0f6010aeb9f35c384a7_0",
//                     "id": "1f7ee1244783e0f6010aeb9f35c384a7",
//                     "name": "…And You Will Know Us by the Trail of Dead"
//                 },
//                 {
//                     "albumCount": 1,
//                     "artistImageUrl": "https://navidrome.moof.vonnieda.org/share/img/eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZCI6ImFyLTBmOWNmZWFkMGEzYWU3NDY4MGQ2MjUzM2I1ODRkMzNkXzAiLCJpc3MiOiJORCJ9.WBVCLKQOavS2Mo6gKKS18DLtxXiH_yjwZT439wTZX4E?size=600",
//                     "coverArt": "ar-0f9cfead0a3ae74680d62533b584d33d_0",
//                     "id": "0f9cfead0a3ae74680d62533b584d33d",
//                     "name": "シートベルツ"
//                 }
//             ],
//             "name": "#"
//         },



// { 
//     id: "677b0f2b8398ecc605be853dbef8db93", 
//     name: "Sixteen Stone", 
//     artist: Some("Bush"), 
//     artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), 
//     cover_id: Some("al-677b0f2b8398ecc605be853dbef8db93_63ebedd0"), 
//     duration: 3173, 
//     year: Some(1994), 
//     genre: Some("Rock"), 
//     song_count: 12, 
//     songs: [
//         Song { 
//             id: "e7e20bbb1299e157f71de5a7ce09c965", 
//             title: "Everything Zen", 
//             album: Some("Sixteen Stone"), 
//             album_id: Some("677b0f2b8398ecc605be853dbef8db93"), 
//             artist: Some("Bush"), 
//             artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), 
//             track: Some(1), 
//             year: Some(1994), 
//             genre: Some("Alternative Rock"), 
//             cover_id: Some("mf-e7e20bbb1299e157f71de5a7ce09c965_63ebedd0"), 
//             size: 9028577, 
//             content_type: "audio/mpeg", 
//             suffix: "mp3", 
//             transcoded_content_type: None, 
//             transcoded_suffix: None, 
//             duration: Some(278), 
//             path: "Bush/Sixteen Stone/01 - Everything Zen.mp3", 
//             media_type: "music", 
//             stream_br: None, 
//             stream_tc: None
//         }, 
//         Song { id: "eaba10fb610d33a13866cb3d8ea18aa0", title: "Swim", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(2), year: Some(1994), genre: Some("Alternative Rock"), cover_id: Some("mf-eaba10fb610d33a13866cb3d8ea18aa0_63ebedd0"), size: 9590019, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(295), path: "Bush/Sixteen Stone/02 - Swim.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "94d780171d889981c3f732c104422164", title: "Bomb", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(3), year: Some(1994), genre: Some("Alternative Rock"), cover_id: Some("mf-94d780171d889981c3f732c104422164_63ebedd0"), size: 6616194, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(202), path: "Bush/Sixteen Stone/03 - Bomb.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "7037cae492bc9226446a16f05c048efa", title: "Little Things", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(4), year: Some(1994), genre: Some("Alternative Rock"), cover_id: Some("mf-7037cae492bc9226446a16f05c048efa_63ebedd0"), size: 8582585, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(264), path: "Bush/Sixteen Stone/04 - Little Things.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "3b682b9d07c61afcecaeb6a687420230", title: "Comedown", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(5), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-3b682b9d07c61afcecaeb6a687420230_63ebedd0"), size: 10583182, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(326), path: "Bush/Sixteen Stone/05 - Comedown.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "88a579bb482230a56444bb7c6e9d9780", title: "Body", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(6), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-88a579bb482230a56444bb7c6e9d9780_63ebedd0"), size: 11204395, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(342), path: "Bush/Sixteen Stone/06 - Body.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "e58e054fc1a42dcd3aa246b98e2c2ce8", title: "Machinehead", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(7), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-e58e054fc1a42dcd3aa246b98e2c2ce8_63ebedd0"), size: 8329809, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(256), path: "Bush/Sixteen Stone/07 - Machinehead.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "f6b88a1eb2b44a7b8309941708908441", title: "Testosterone", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(8), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-f6b88a1eb2b44a7b8309941708908441_63ebedd0"), size: 8437308, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(259), path: "Bush/Sixteen Stone/08 - Testosterone.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "f1d5fefd97322aa8c3e647bb95fb5b67", title: "Monkey", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(9), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-f1d5fefd97322aa8c3e647bb95fb5b67_63ebedd0"), size: 7830649, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(240), path: "Bush/Sixteen Stone/09 - Monkey.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "7ec8698d0e6d086780d2b3fb5863b3af", title: "Glycerine", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(10), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-7ec8698d0e6d086780d2b3fb5863b3af_63ebedd0"), size: 8654021, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(266), path: "Bush/Sixteen Stone/10 - Glycerine.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "9ae51540db40bc6c736ab1dd4656ba27", title: "Alien", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(11), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-9ae51540db40bc6c736ab1dd4656ba27_63ebedd0"), size: 12749089, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(394), path: "Bush/Sixteen Stone/11 - Alien.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }, Song { id: "c314baf42c1b3050e5854e8946f53143", title: "X‐Girlfriend", album: Some("Sixteen Stone"), album_id: Some("677b0f2b8398ecc605be853dbef8db93"), artist: Some("Bush"), artist_id: Some("dbbc546e357040b4a83a65f387d5c06a"), track: Some(12), year: Some(1994), genre: Some("Rock"), cover_id: Some("mf-c314baf42c1b3050e5854e8946f53143_63ebedd0"), size: 1561682, content_type: "audio/mpeg", suffix: "mp3", transcoded_content_type: None, transcoded_suffix: None, duration: Some(45), path: "Bush/Sixteen Stone/12 - X‐Girlfriend.mp3", media_type: "music", stream_br: None, stream_tc: None
//         }
//     ]
// }