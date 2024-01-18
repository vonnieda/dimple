use std::default;

use dimple_core::{library::{Library, LibraryEntity, LibrarySupport}, model::{DimpleRelationContent, DimpleArtist, Attributed, DimpleRelation}};
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

impl WikidataLibrary {
    fn get_summary(&self, relations: &Vec<DimpleRelation>) -> Option<String> {
        // Find a Wikidata link if one exists.
        let wikidata_url: String = relations
            .iter()
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

        // Get the English Wikipedia URL for the item
        fn non_empty(s: &String) -> Option<String> {
            if s.is_empty() {
                None
            }
            else {
                Some(s.to_string())
            }
        }


        // TODO support languange priority, this is temp
        let wikipedia_url = non_empty(&wikidata_item.sitelinks.enwiki.url)
            .or(non_empty(&wikidata_item.sitelinks.eswiki.url))
            .or(non_empty(&wikidata_item.sitelinks.frwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.dewiki.url))
            .or(non_empty(&wikidata_item.sitelinks.ptwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.itwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.ruwiki.url))
            .or(non_empty(&wikidata_item.sitelinks.svwiki.url))
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
        let client = Client::builder()
            // .https_only(true)
            .user_agent(dimple_core::USER_AGENT)
            .build().ok()?;
        let url = format!("https://en.wikipedia.org/api/rest_v1/page/summary/{}", wikipedia_title);
        LibrarySupport::log_request(self, &url);
        let response = client.get(url).send().ok()?;
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
// TODO can also get images here.
// TODO this is a hyperpanic mess. Will be rewritten. I WANT BIOS
impl Library for WikidataLibrary {
    fn name(&self) -> String {
        "Wikidata".to_string()
    }

    fn fetch(&self, entity: &LibraryEntity) -> Option<LibraryEntity> {
        match entity.clone() {
            LibraryEntity::Artist(mut artist) => {
                artist.summary = self.get_summary(&artist.relations).unwrap_or_default();
                Some(LibraryEntity::Artist(artist))
            },

            LibraryEntity::ReleaseGroup(mut release_group) => {
                release_group.summary = self.get_summary(&release_group.relations)
                    .unwrap_or_default();
                Some(LibraryEntity::ReleaseGroup(release_group))
            },

            _ => None,
        }
    }
}
