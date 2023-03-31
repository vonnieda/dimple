use std::{io::{Cursor}, sync::Arc};

use config::Config;
use crossbeam::channel::{Receiver, unbounded};
use image::DynamicImage;
use log::{debug};
use rodio::{Decoder, Sink};
use sunk::{Client, search::SearchPage, ListType, Album, Media, song::Song, Streamable};
use url::Url;

use super::{Library, Release, Artist, Image, Genre, Track, image_cache::ImageCache};

use std::iter::Iterator;

use rayon::prelude::*;

use std::thread;

pub struct NavidromeLibrary {
    site: String,
    username: String,
    password: String,
    image_cache: ImageCache,
}

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
                // For each artist get all the albums
                .flat_map(|artist| artist.albums(&client).unwrap())
                // For each album convert to a release and send
                .map(|album| Self::album_to_release(&base_url, &album))
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

    fn album_to_release(base_url: &str, album: &Album) -> Release {
        let artists = album.artist.as_ref().map_or(vec![], |artist| {
            vec![Artist {
                // TODO need ID
                url: Self::url(base_url, "artist", artist),
                name: artist.to_string(),
                // TODO get artist art
                art: vec![],
            }]
        });
        let tracks: Vec<Track> = album.songs
            .iter()
            .map(|song| {
                Track {
                    url: Self::url(base_url, "track", &song.id),
                    title: song.title.clone(),
                    ..Default::default() 
                }
            })
            .collect();

        let art = album.cover_id.as_ref()
            .map_or(None, |cover_id| Some(Image {
                url: Self::url(base_url, "image", cover_id),
            }))
            .map_or(vec![], |image| vec![image]);


        let genres = album.genre.as_ref().map_or(vec![], |genre| 
            vec![Genre {
                // TODO ID
                url: Self::url(base_url, "genre", &genre),
                name: genre.clone(),
                art: vec![],
            }]
        );
        Release {
            url: Self::url(base_url, "release", &album.id),
            title: album.name.clone(),
            artists: artists,
            tracks: tracks,
            art: art,
            genres: genres,
        }
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
