use std::{fs};

use bincode::de;
use config::Config;
use log::{debug, info};
use rayon::prelude::*;
use sunk::{Client, search::SearchPage, ListType, Error, Album};
use url::Url;

use super::{Library, Release, Artist, Image, Genre, Track};

const CACHE_DIR: &str = "data/navidrome/images/original";

// TODO looks like getIndexes might be how to check for changes?
// http://www.subsonic.org/pages/api.jsp#getIndexes

pub struct NavidromeLibrary {
    // TODO add path, or just take a sled?
    site: String,
    username: String,
    password: String,
}

impl NavidromeLibrary {
    pub fn new(site: &str, username: &str, password: &str) -> Self {
        Self {
            site: String::from(site),
            username: String::from(username),
            password: String::from(password),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self {
            site: config.get_string("navidrome.site").unwrap(),
            username: config.get_string("navidrome.username").unwrap(),
            password: config.get_string("navidrome.password").unwrap(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("navidrome://{}@{}", self.username, path)
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

    fn album_to_release(album: &Album) -> Release {
        let UNKNOWN_ARTIST:Artist = Artist {
            url: "navidrome://artist/UNKNOWN".to_string(),
            name: "UNKNOWN".to_string(),
            ..Default::default()
        };
        Release {
            url: "".to_string(),
            title: album.name.clone(),
            artists: vec![album.artist.as_ref().map_or(
                UNKNOWN_ARTIST,
                |artist| Artist {
                    name: artist.to_string(),
                    ..Default::default()
                })],
            ..Default::default()
        }

        // pub struct Release {
        //     pub url: String,
        //     pub title: String,
        //     pub artists: Vec<Artist>,
        //     pub art: Vec<Image>,
        //     pub genres: Vec<Genre>,
        //     pub tracks: Vec<Track>,
        // }
        
        


    }

    // fn get_release(&self, album: &Album) -> Result<Release, Error> {
    //     let client = self.new_client()?;
    //     album.get
    // }
}

impl Library for NavidromeLibrary {
    /// Get all albums as releases. Fully populates all objects by calling
    /// getAlbum for each.
    fn releases(self: &Self) -> Result<Vec<Release>, String> {
        // TODO this chain could populate the artists too by branching out
        // to fetch their art and such. Or maybe better to just drop artist
        // for now and go with Strings.
        let client = self.new_client().map_err(|err| err.to_string())?;
        let albums = self.get_all_albums()
            .map_err(|err| err.to_string())?
            .par_iter()
            // .inspect(|shallow_album| println!("{:?}", shallow_album))
            .map(|shallow_album| {
                Album::get(&client, &shallow_album.id).map_err(|err| err.to_string())
            })
            // .inspect(|deep_album| println!("{:?}", deep_album))
            .collect::<Result<Vec<Album>, String>>()?;
        Ok(albums.par_iter()
            .map(|album| Self::album_to_release(album))
            .collect::<Vec<Release>>())
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
