// TODO Check out anyhow https://docs.rs/anyhow/latest/anyhow/

use std::{sync::{Arc, mpsc::Receiver}};

use data_encoding::{BASE64};
use image::DynamicImage;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sunk::{Client, Album, Media, song::Song, Streamable};
use threadpool::ThreadPool;
use url::Url;

use super::{Library, Release, Image, Track, image_cache::ImageCache};

use std::iter::Iterator;

use std::thread;

pub struct NavidromeLibrary {
    pub ulid: String,
    pub name: String,
    pub site: String,
    pub username: String,
    pub password: String,
    image_cache: Option<ImageCache>,
}

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct NavidromeConfig {
    pub ulid: String,
    pub name: String,
    pub site: String,
    pub username: String,
    pub password: String,
}

impl From<NavidromeConfig> for NavidromeLibrary {
    fn from(config: NavidromeConfig) -> Self {
        Self::new(&config.ulid, 
            &config.name, 
            &config.site, 
            &config.username, 
            &config.password)
    }
}

impl Library for NavidromeLibrary {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn releases(&self) -> Receiver<Release> {
        let client = Arc::new(Box::new(self.new_client().unwrap()));
        let (sender, receiver) = std::sync::mpsc::channel::<Release>();
        let base_url = self.base_url();

        thread::spawn(move || {
            let sender = sender.clone();

            let pool = ThreadPool::default();

            for artist in sunk::Artist::list(&client, 0).unwrap() {
                let sender = sender.clone();
                let client = client.clone();
                let base_url = base_url.clone();
                pool.execute(move || {
                    let pool = ThreadPool::default();
                    for shallow_album in artist.albums(&client.clone()).unwrap() {
                        let client = client.clone();
                        let base_url = base_url.clone();
                        let artist = artist.clone();
                        let sender = sender.clone();
                        pool.execute(move || {
                            let album = Album::get(&client, &shallow_album.id).unwrap();
                            let release = Self::sunk_artist_album_to_dimple_release(&base_url, 
                                &artist, 
                                &album);
                            sender.send(release).unwrap();
                        });
                    }
                });
            }
        });

        receiver
    }

    fn image(&self, image: &Image) -> Result<DynamicImage, String> {
        let (_object_type, id) = self.un_url(&image.url);

        let cache = self.image_cache.as_ref().unwrap();

        // check cache first
        if let Some(image) = cache.get_original(&id) {
            return Ok(image);
        }

        // not found, download it
        let album = sunk::Album {
            cover_id: Some(id.to_string()),
            ..Default::default()
        };
        log::debug!("Downloading {}", id);
        let client = self.new_client()?;
        let bytes = album.cover_art(&client, 0).map_err(|e| e.to_string())?;
        let dynamic_image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;

        // cache it
        cache.insert(&id, &dynamic_image);

        // return it
        Ok(dynamic_image)
    }

    fn stream(&self, track: &Track) -> Result<Vec<u8>, String> {
        let (_object_type, id) = self.un_url(&track.url);
        let client = self.new_client()?;
        let song = Song::get(&client, &id).unwrap();
        song.stream(&client).map_err(|err| err.to_string())
    }
}

impl NavidromeLibrary {
    pub fn new(ulid: &str, name: &str, site: &str, username: &str, password: &str) -> Self {
        // TODO don't love this hardcoded path.
        // TODO get library type in path, the ulids are annoying on disk.
        // TODO this is slow, about 800ms. 
        let db = sled::open(format!("data/{}", ulid)).unwrap();
        Self {
            ulid: String::from(ulid),
            name: String::from(name),
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
            image_cache: Some(ImageCache::new(db.open_tree("image_cache").unwrap())),
        }
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
        (object_type, id)
    }

    fn new_client(&self) -> Result<Client, String> {
        sunk::Client::new(
            self.site.as_str(),
            self.username.as_str(),
            self.password.as_str(),
        ).map_err(|e| e.to_string())
    }

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
    fn sunk_artist_album_to_dimple_release(base_url: &str, 
        artist: &sunk::Artist,
        album: &sunk::Album) -> crate::music_library::Release {
            let url = Self::url(base_url, "album", &album.id); 
            let title = album.name.clone();
            let artists = vec![
                Self::sunk_artist_to_dimple_artist(base_url, artist)
            ]; 
            let tracks = album.songs.iter().map(|song| {
                Self::sunk_song_to_dimple_track(base_url, song)
            }).collect();
            let art = album.cover_id().map_or(
                vec![], 
                |cover_id| vec![Self::sunk_image_to_dimple_image(base_url, cover_id)]
            );
            let genres = album.genre.as_ref().map_or(
                vec![], 
                |genre_name| vec![Self::sunk_genre_to_dimple_genre(base_url, genre_name)]
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

    // http://your-server/rest/getArtist
    // <subsonic-response status="ok" version="1.8.0">
    // <artist id="5432" name="AC/DC" coverArt="ar-5432" albumCount="15">
    // <album id="11047" name="Back In Black" coverArt="al-11047" songCount="10" created="2004-11-08T23:33:11" duration="2534" artist="AC/DC" artistId="5432"/>
    // ..
    // <album id="11061" name="Who Made Who" coverArt="al-11061" songCount="9" created="2004-11-08T23:43:18" duration="2291" artist="AC/DC" artistId="5432"/>
    // </artist>
    // </subsonic-response>
    fn sunk_artist_to_dimple_artist(base_url: &str, artist: &sunk::Artist) -> crate::music_library::Artist {
        let art = artist.cover_id().map_or(
            vec![], 
            |cover_id| vec![Self::sunk_image_to_dimple_image(base_url, cover_id)]
        );
        crate::music_library::Artist {
            url: Self::url(base_url, "artist", &artist.id),
            name: artist.name.clone(),
            art,
            ..Default::default()
        }
    }

    fn sunk_image_to_dimple_image(base_url: &str, image_id: &str) -> crate::music_library::Image {
        crate::music_library::Image {
            url: Self::url(base_url, "image", image_id)
        }
    }

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
    fn sunk_genre_to_dimple_genre(base_url: &str, genre_name: &str) -> crate::music_library::Genre {
        crate::music_library::Genre {
            url: Self::url(base_url, "genre", &Self::string_to_id(genre_name)),
            name: genre_name.to_string(),
            art: vec![],
        }
    }

    // http://your-server/rest/getSong
    // <subsonic-response status="ok" version="1.8.0">
    // <song id="48228" parent="48203" title="You Shook Me All Night Long" album="Back In Black" artist="AC/DC" isDir="false" coverArt="48203" created="2004-11-08T23:33:11" duration="210" bitRate="112" size="2945619" suffix="mp3" contentType="audio/mpeg" isVideo="false" path="ACDC/Back in black/ACDC - You Shook Me All Night Long.mp3"/>
    // </subsonic-response>
    fn sunk_song_to_dimple_track(base_url: &str, song: &sunk::song::Song) -> crate::music_library::Track {
        Track {
            url: Self::url(base_url, "track", &song.id),
            title: song.title.clone(),
            ..Default::default() 
        }
    }

    fn string_to_id(input: &str) -> String {
        BASE64.encode(&Sha256::digest(input))
    }
}

