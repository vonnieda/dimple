use std::collections::HashSet;

use anyhow::Result;
use dimple_core::model::{Entity, Artist, Model, Recording, Release, ReleaseGroup};
use dimple_librarian::plugin::{LibrarySupport, Plugin};
use reqwest::{blocking::Client, Url};
use serde::Deserialize;

#[derive(Debug, Default)]
pub struct WikidataPlugin {
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
    descriptions: WdDescription,
}

// https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdSiteLinks {
    cswiki: WdSiteLink,
    dawiki: WdSiteLink,
    dewiki: WdSiteLink,
    enwiki: WdSiteLink,
    eswiki: WdSiteLink,
    frwiki: WdSiteLink,
    itwiki: WdSiteLink,
    jawiki: WdSiteLink,
    nowiki: WdSiteLink,
    ptwiki: WdSiteLink,
    ruwiki: WdSiteLink,
    svwiki: WdSiteLink,
    vlwiki: WdSiteLink,
    commonswiki: WdSiteLink,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdDescription {
    de: String, 
    en: String,
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

// TODO expand this to pull in all the alternate IDs and store them on objects.
// https://www.wikidata.org/wiki/Q2549534
impl WikidataPlugin {
    fn get_summary(&self, links: &HashSet<String>) -> Option<String> {
        // Find a Wikidata link if one exists.
        let wikidata_url = links.iter()
            .find_map(|link| {
                if link.starts_with("https://www.wikidata.org/wiki/Q") || link.starts_with("http://www.wikidata.org/wiki/Q") {
                    return Some(link.clone());
                }
                None
            })?;
            
        // Extract the Wikidata ID
        let parsed_url = Url::parse(&wikidata_url).ok()?;
        let wikidata_id = parsed_url.path_segments()?.nth(1)?;

        // Use the Wikidata API to fetch the item
        let client = Client::builder()
            .user_agent(dimple_librarian::plugin::USER_AGENT)
            .build().ok()?;
        let url = format!("https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/{}", wikidata_id);
        let request_token = LibrarySupport::start_request(self, &url);
        let response = client.get(url).send().ok()
            .inspect(|f| {
                LibrarySupport::end_request(request_token, 
                    Some(f.status().as_u16()), 
                    f.content_length());
            })?;
        let wikidata_item = response.json::<WdItem>().ok()?;

        fn non_empty(s: &String) -> Option<String> {
            if s.is_empty() {
                None
            }
            else {
                Some(s.to_string())
            }
        }


        // Get the Wikipedia URL for the item
        // TODO support language priority, this is temp
        let wikipedia_url = non_empty(&wikidata_item.sitelinks.enwiki.url)
            .or(non_empty(&wikidata_item.sitelinks.eswiki.url))
            .or(non_empty(&wikidata_item.sitelinks.frwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.dewiki.url))
            .or(non_empty(&wikidata_item.sitelinks.ptwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.itwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.ruwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.svwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.nowiki.url))
            .or(non_empty(&wikidata_item.sitelinks.jawiki.url))
            .or(non_empty(&wikidata_item.sitelinks.cswiki.url))
            .or(non_empty(&wikidata_item.sitelinks.dawiki.url))
            .or(non_empty(&wikidata_item.sitelinks.vlwiki.url))
            ?;

        // Extract the Wikipedia title
        // https://en.wikipedia.org/wiki/Brutus_(Belgian_band)
        let parsed_url = Url::parse(&wikipedia_url).ok()?;
        let wikipedia_title = parsed_url.path_segments()?.nth(1)?;

        // Use the Wikipedia API to fetch the summary
        // TODO should wikipedia be it's own library, perhaps after the ids have been extracted?
        let client = Client::builder()
            // .https_only(true)
            .user_agent(dimple_librarian::plugin::USER_AGENT)
            .build().ok()?;
        let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", wikipedia_title);
        let request_token = LibrarySupport::start_request(self, &url);
        let response = client.get(url).send().ok()
            .inspect(|f| {
                LibrarySupport::end_request(request_token, 
                    Some(f.status().as_u16()), 
                    f.content_length());
            })?;

        let wikipedia_summary = response.json::<WpSummary>().ok()?;

        if wikipedia_summary.extract.is_empty() {
            return None;
        }

        Some(wikipedia_summary.extract)
    }
}

// https://www.wikidata.org/wiki/Wikidata:REST_API
// https://stackoverflow.com/questions/8555320/is-there-a-wikipedia-api-just-for-retrieve-the-content-summary
// https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts&exintro&explaintext&redirects=1&pageids=21721040
// https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/Q1514317
// https://www.wikidata.org/w/rest.php/wikibase/v0/entities/items/Q30335935
// sitelinks.enwiki https://en.wikipedia.org/wiki/Brutus_(Belgian_band)
// https://en.wikipedia.org/api/rest_v1/page/summary/Brutus_(Belgian_band)                
// https://en.wikipedia.org/w/api.php?format=json&action=query&prop=extracts&exintro&explaintext&redirects=1&titles=Stack%20Overflow                
// TODO can also get images here and wiki commons? or via here to wiki commons?
impl Plugin for WikidataPlugin {
    fn name(&self) -> String {
        "Wikidata".to_string()
    }

    fn get(&self, entity: &dyn Entity, network_mode: dimple_librarian::plugin::NetworkMode) -> Result<Option<Box<dyn Entity>>> {
        let model = entity.model();
        match model {
            Model::Artist(artist) => {
                let artist = self.get_summary(&artist.links)
                    .map(|summary| Artist {
                        summary: Some(summary),
                        ..Default::default()
                    });
                if let Some(artist) = artist {
                    Ok(Some(Box::new(artist)))
                }
                else {
                    Ok(None)
                }
            },
            _ => Ok(None),
        }
    }
}


// fn fetch(&self, entity: &Entities) -> Option<Entities> {
//     match entity.clone() {
//         Entities::Artist(artist) => self.get_summary(&artist.links)
//             .map(|summary| Artist {
//                 summary: Some(summary),
//                 ..Default::default()
//             }.entity()),
            
//         Entities::ReleaseGroup(rg) => self.get_summary(&rg.links)
//             .map(|summary| ReleaseGroup {
//                 summary: Some(summary),
//                 ..Default::default()
//             }.entity()),

//         Entities::Release(rg) => self.get_summary(&rg.links)
//             .map(|summary| Release {
//                 summary: Some(summary),
//                 ..Default::default()
//             }.entity()),

//         Entities::Recording(r) => self.get_summary(&r.links)
//             .map(|summary| Recording {
//                 summary: Some(summary),
//                 ..Default::default()
//             }.entity()),

//         _ => None,
//     }
// }
    
