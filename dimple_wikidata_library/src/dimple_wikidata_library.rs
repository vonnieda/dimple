use std::default;

use dimple_core::{library::{Library, LibraryEntity, LibrarySupport}, model::{DimpleRelationContent, DimpleArtist, Attributed}};
use reqwest::{blocking::Client, Url};
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct WikidataLibrary {
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdItem {
    #[serde(rename="type")]
    typeh: String,
    sitelinks: WdSiteLinks,
    // artistthumb: Vec<ImageResponse>,
    // musiclogo: Vec<ImageResponse>,
    // hdmusiclogo: Vec<ImageResponse>,
    // artistbackground: Vec<ImageResponse>,
    // status: String,
    // error_message: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdSiteLinks {
    ptwiki: WdSiteLink,
    dewiki: WdSiteLink,
    jawiki: WdSiteLink,
    cswiki: WdSiteLink,
    frwiki: WdSiteLink,
    ruwiki: WdSiteLink,
    eswiki: WdSiteLink,
    itwiki: WdSiteLink,
    svwiki: WdSiteLink,
    vlwiki: WdSiteLink,
    dawiki: WdSiteLink,
    enwiki: WdSiteLink,
    commonswiki: WdSiteLink,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdSiteLink {
    title: String,
    url: String,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WpSummary {
    extract: String,
}

// https://www.wikidata.org/wiki/Wikidata:REST_API
// https://stackoverflow.com/questions/8555320/is-there-a-wikipedia-api-just-for-retrieve-the-content-summary
// https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts&exintro&explaintext&redirects=1&pageids=21721040
// https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/Q1514317
// TODO can also get images here.
// TODO this is a hyperpanic mess. Will be rewritten. I WANT BIOS
impl Library for WikidataLibrary {
    fn name(&self) -> String {
        "Wikidata".to_string()
    }

    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        match entity {
            LibraryEntity::Artist(artist) => {
                // https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/Q30335935
                // sitelinks.enwiki https://en.wikipedia.org/wiki/Brutus_(Belgian_band)
                // https://en.wikipedia.org/api/rest_v1/page/summary/Brutus_(Belgian_band)                
                // https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts&exintro&explaintext&redirects=1&titles=Stack%20Overflow                

                // Find a Wikidata link if one exists.
                let wikidata_url: String = artist.relations
                    .iter()
                    .flatten()
                    .map(|rel| rel.to_owned())
                    .filter_map(|rel| match rel.content {
                        DimpleRelationContent::Url(url) => Some(url.resource),
                        _ => None,
                    })
                    .find(|f| f.starts_with("https://www.wikidata.org/wiki/Q"))?;

                // Extract the Wikidata ID
                let parsed_url = Url::parse(&wikidata_url).ok()?;
                let wikidata_id = parsed_url.path_segments()?.nth(1)?;

                // Use the Wikidata API to fetch the item
                let client = Client::builder()
                    // .https_only(true)
                    .user_agent(dimple_core::USER_AGENT)
                    .build().ok()?;
                let url = format!("https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/{}", wikidata_id);
                LibrarySupport::log_request(self, &url);
                let response = client.get(url).send().ok()?;
                let wikidata_item = response.json::<WdItem>().ok()?;

                if !wikidata_item.sitelinks.enwiki.url.is_empty() {
                    // Extract the Wikipedia title
                    // https://en.wikipedia.org/wiki/Brutus_(Belgian_band)
                    let parsed_url = Url::parse(&wikidata_item.sitelinks.enwiki.url).ok()?;
                    let wikipedia_title = parsed_url.path_segments()?.nth(1)?;

                    // Use the Wikipedia API to fetch the summary
                    let client = Client::builder()
                        // .https_only(true)
                        .user_agent(dimple_core::USER_AGENT)
                        .build().ok()?;
                    let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", wikipedia_title);
                    LibrarySupport::log_request(self, &url);
                    let response = client.get(url).send().ok()?;
                    let wikipedia_summary = response.json::<WpSummary>().ok()?;

                    if !wikipedia_summary.extract.is_empty() {
                        let mut artist = artist.clone();
                        artist.bio = Some(Attributed {
                            value: wikipedia_summary.extract,
                            ..Default::default()
                        });
                        return Some(LibraryEntity::Artist(artist))
                    }
                }

                None
            },

            _ => None,
        }
    }
}


            // // https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/Q1514317
            // let id = Url::parse(&url).ok()
            //     .and_then(|url| {
            //         let segs = url.path_segments();
            //     }
            //     .and_then(|f| f.nth(1));

            // let client = Client::builder()
            //     // .https_only(true)
            //     .user_agent(dimple_core::USER_AGENT)
            //     .build().ok()?;
            // LibrarySupport::log_request(self, &url);
            // let response = client.get(url).send().unwrap();
            // let item_resp = response.json::<WdItem>().unwrap();
            // log::info!("Found {:?}", item_resp);
            // None
            // },




// let client = Client::builder()
// .https_only(true)
// .user_agent(dimple_core::USER_AGENT)
// .build().ok()?;
// let api_key = "55b9ef19f6822b9f835c97426d435d72";
// let mbid = a.id.to_string();
// let url = format!("https://webservice.fanart.tv/v3/music/{}?api_key={}", mbid, api_key);
// LibrarySupport::log_request(self, &url);
// let response = client.get(url).send().ok()?;
// let artist_resp = response.json::<ArtistResponse>().ok()?;
// let thumb = artist_resp.artistthumb.first()
// .or_else(|| artist_resp.artistbackground.first())
// .or_else(|| artist_resp.hdmusiclogo.first())
// .or_else(|| artist_resp.musiclogo.first())?;
// LibrarySupport::log_request(self, &thumb.url);
// let thumb_resp = client.get(&thumb.url).send().ok()?;
// let bytes = thumb_resp.bytes().ok()?;
// image::load_from_memory(&bytes).ok()
