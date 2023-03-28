use config::Config;
use image::DynamicImage;
use log::{debug};
use rayon::prelude::*;
use sunk::{Client, search::SearchPage, ListType, Album, Media};
use url::Url;

use super::{Library, Release, Artist, Image, Genre, Track, image_cache::ImageCache};


// TODO looks like getIndexes might be how to check for changes?
// http://www.subsonic.org/pages/api.jsp#getIndexes
// TODO add an initial quick scan that returns early if we already have all
// the albums exactly the same. Maybe just hash it.
pub struct NavidromeLibrary {
    site: String,
    username: String,
    password: String,
    image_cache: ImageCache,
}

impl Library for NavidromeLibrary {
    fn releases(self: &Self) -> Result<Vec<Release>, String> {
        let client = self.new_client().map_err(|err| err.to_string())?;
        let releases = self.get_all_albums()
            .map_err(|err| err.to_string())?
            .par_iter()
            .map(|shallow_album| {
                Album::get(&client, &shallow_album.id)
                    .map_err(|err| err.to_string())
            })
            .collect::<Result<Vec<Album>, String>>()?
            .par_iter()
            .map(|album| self.album_to_release(album))
            // .inspect(|release| println!("{:?}", release))
            .collect::<Vec<Release>>();
        Ok(releases)
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

    fn url(&self, object_type: &str, id: &str) -> String {
        format!("navidrome:///{}/{}/{}", self.username, object_type, id)
    }

    fn _un_url(&self, url: &str) -> (String, String) {
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

    fn get_albums(&self, count: usize, offset: usize) -> Result<Vec<Album>, String> {
        debug!("getting albums {} through {}", offset, offset + count - 1);
        let page = SearchPage { count, offset };
        let list_type = ListType::default();
        let client = self.new_client()?;
        sunk::Album::list(&client, list_type, page, 0)
            .map_err(|e| e.to_string())
    }

    fn get_all_albums(&self) -> Result<Vec<Album>, String> {
        let mut all_albums: Vec<Album> = Vec::new();
        let mut page = SearchPage {
            count: 500,
            offset: 0,
        };
        loop {
            // TODO ugly
            if let Ok(albums) = self.get_albums(page.count, page.offset) {
                if albums.len() == 0 {
                    break;
                }
                all_albums.extend(albums);
                page.offset += page.count;
            }
            else {
                break;
            }
        }
        Ok(all_albums)
    }

    fn album_to_release(&self, album: &Album) -> Release {
        let artists = album.artist.as_ref().map_or(vec![], |artist| {
            vec![Artist {
                // TODO need ID
                url: self.url("artist", artist),
                name: artist.to_string(),
                // TODO get artist art
                art: vec![],
            }]
        });
        let tracks: Vec<Track> = album.songs
            .par_iter()
            .map(|song| {
                Track {
                    url: self.url("track", &song.id),
                    title: song.title.clone(),
                    ..Default::default() 
                }
            })
            .collect();

        let art = album.cover_id.as_ref()
            .map_or(None, |cover_id| self.get_image(&cover_id).map_or_else(|_| None, |image| Some((cover_id, image))))
            .map_or(None, |(cover_id, image)| Some(Image {
                url: self.url("image", cover_id),
                original: image,
            }))
            .map_or(None, |image| Some(image))
            .map_or(vec![], |image| vec![image]);


        let genres = album.genre.as_ref().map_or(vec![], |genre| 
            vec![Genre {
                // TODO ID
                url: self.url("genre", &genre),
                name: genre.clone(),
                art: vec![],
            }]
        );
        Release {
            url: self.url("release", &album.id),
            title: album.name.clone(),
            artists: artists,
            tracks: tracks,
            art: art,
            genres: genres,
        }
    }

    fn get_image(&self, id: &str) -> Result<DynamicImage, String> {
        // check cache first
        if let Some(image) = self.image_cache.get_original(id) {
            return Ok(image);
        }

        // not found, download it
        let album = sunk::Album {
            id: Default::default(),
            name: Default::default(),
            artist: Default::default(),
            artist_id: Default::default(),
            duration: Default::default(),
            song_count: Default::default(),
            songs: Default::default(),
            year: Default::default(),
            genre: Default::default(),
            cover_id: Some(id.to_string()),
        };
        let client = self.new_client()?;
        debug!("Downloading {}", id);
        let bytes = album.cover_art(&client, 0).map_err(|e| e.to_string())?;
        let dynamic_image = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;

        // cache it
        self.image_cache.insert(id, &dynamic_image);

        // return it
        return Ok(dynamic_image);
    }
}


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
