use reqwest::Url;
use serde::Deserialize;

use crate::{librarian::ArtistMetadata, library::Library, model::Artist};

use super::{plugin::Plugin, plugins::{nempty, Plugins}};

impl Plugin for WikidataPlugin {
    fn type_name(&self) -> String {
        "WikiDataPlugin".to_string()
    }

    fn display_name(&self) -> String {
        "Wikidata".to_string()
    }
    
    fn artist_metadata(&self, host: &Plugins, library: &Library, artist: &crate::model::Artist) -> Result<Option<crate::librarian::ArtistMetadata>, anyhow::Error> {
        let client = WikidataClient::default();
        let links = artist.links(library).iter().map(|l| l.url.to_string()).collect::<Vec<_>>();
        
        // First try with existing links
        if let Some(summary) = client.get_summary(&links, host) {
            return Ok(Some(Self::create_artist_metadata(summary, None)));
        }
        
        // Fallback: try looking up by MusicBrainz ID if available
        if let Some(ref musicbrainz_id) = artist.musicbrainz_id {
            if let Some((summary, wikidata_url)) = client.get_summary_by_musicbrainz_id(musicbrainz_id, host) {
                // Only include the found URL if there's no existing wikidata_id
                let url_to_include = if artist.wikidata_id.is_none() {
                    Some(wikidata_url)
                } else {
                    None
                };
                
                return Ok(Some(Self::create_artist_metadata(summary, url_to_include)));
            }
        }
        
        Ok(None)
    }
    
    fn track_metadata(&self, _host: &Plugins, _library: &crate::library::Library, _track: &crate::model::Track) -> Result<Option<crate::librarian::TrackMetadata>, anyhow::Error> {
        Ok(None)
    }
    
    fn release_metadata(&self, _host: &Plugins, _library: &crate::library::Library, _release: &crate::model::Release) -> Result<Option<crate::librarian::ReleaseMetadata>, anyhow::Error> {
        Ok(None)
    }    

    fn release_group_metadata(&self, _host: &Plugins, _library: &crate::library::Library, _release_group: &crate::model::ReleaseGroup) -> Result<Option<crate::librarian::ReleaseGroupMetadata>, anyhow::Error> {
        // TODO
        Ok(None)
    }    
}

impl WikidataPlugin {
    fn create_artist_metadata(summary: String, wikidata_url: Option<String>) -> ArtistMetadata {
        ArtistMetadata {
            artist: Artist {
                summary: Some(summary),
                wikidata_id: wikidata_url,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Default)]
pub struct WikidataPlugin {
} 

#[derive(Default)]
struct WikidataClient {

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
impl WikidataClient {
    const MUSICBRAINZ_PROPERTY: &'static str = "P434";
    const WIKIDATA_API_BASE: &'static str = "https://www.wikidata.org/w/api.php";
    const WIKIDATA_REST_BASE: &'static str = "https://www.wikidata.org/w/rest.php/wikibase/v1/entities/items";
    const WIKIDATA_BASE_URL: &'static str = "https://www.wikidata.org/wiki";
    const WIKIPEDIA_API_BASE: &'static str = "https://en.wikipedia.org/api/rest_v1/page/summary";

    fn get_summary_by_musicbrainz_id(&self, musicbrainz_id: &str, host: &Plugins) -> Option<(String, String)> {
        // Query Wikidata for items with the given MusicBrainz artist ID (P434)
        let search_url = format!(
            "{}?action=query&format=json&list=search&srsearch=haswbstatement:{}={}",
            Self::WIKIDATA_API_BASE, Self::MUSICBRAINZ_PROPERTY, musicbrainz_id
        );
        
        let response = host.get(&search_url).ok()?;
        let search_result = response.json::<WdSearchResponse>().ok()?;
        
        if let Some(first_result) = search_result.query.search.first() {
            let wikidata_id = &first_result.title;
            let wikidata_url = format!("{}/{}", Self::WIKIDATA_BASE_URL, wikidata_id);
            
            // Now get the summary using the found Wikidata ID
            if let Some(summary) = self.get_summary_by_wikidata_id(wikidata_id, host) {
                return Some((summary, wikidata_url));
            }
        }
        
        None
    }
    
    fn get_summary_by_wikidata_id(&self, wikidata_id: &str, host: &Plugins) -> Option<String> {
        // Use the Wikidata API to fetch the item
        let url = format!("{}/{}", Self::WIKIDATA_REST_BASE, wikidata_id);
        let response = host.get(&url).ok()?;
        let wikidata_item = response.json::<WdItem>().ok()?;

        // Get the Wikipedia URL for the item using language priority
        let wikipedia_url = self.find_wikipedia_url(&wikidata_item.sitelinks)?;

        // Extract the Wikipedia title
        let parsed_url = Url::parse(&wikipedia_url).ok()?;
        let wikipedia_title = parsed_url.path_segments()?.nth(1)?;

        // Use the Wikipedia API to fetch the summary
        self.fetch_wikipedia_summary(wikipedia_title, host)
    }

    /// Find the first available Wikipedia URL from sitelinks, using language priority
    fn find_wikipedia_url(&self, sitelinks: &WdSiteLinks) -> Option<String> {
        // Language priority: English first, then major European languages, then others
        let language_priority = [
            &sitelinks.enwiki,    // English
            &sitelinks.eswiki,    // Spanish
            &sitelinks.frwiki,    // French
            &sitelinks.dewiki,    // German
            &sitelinks.ptwiki,    // Portuguese
            &sitelinks.itwiki,    // Italian
            &sitelinks.ruwiki,    // Russian
            &sitelinks.svwiki,    // Swedish
            &sitelinks.nowiki,    // Norwegian
            &sitelinks.jawiki,    // Japanese
            &sitelinks.cswiki,    // Czech
            &sitelinks.dawiki,    // Danish
            &sitelinks.vlwiki,    // Flemish
        ];

        for site_link in language_priority {
            if let Some(url) = nempty(&site_link.url) {
                return Some(url);
            }
        }
        None
    }

    /// Fetch Wikipedia summary for a given article title
    fn fetch_wikipedia_summary(&self, wikipedia_title: &str, host: &Plugins) -> Option<String> {
        let url = format!("{}/{}", Self::WIKIPEDIA_API_BASE, wikipedia_title);
        let response = host.get(&url).ok()?;
        let wikipedia_summary = response.json::<WpSummary>().ok()?;
        
        if wikipedia_summary.extract.is_empty() {
            None
        } else {
            Some(wikipedia_summary.extract)
        }
    }

    fn get_summary(&self, links: &[String], host: &Plugins) -> Option<String> {
        // Find a Wikidata link if one exists.
        let wikidata_id = self.extract_wikidata_id_from_links(links)?;
        
        // Use the refactored method to get the summary
        self.get_summary_by_wikidata_id(&wikidata_id, host)
    }

    /// Extract Wikidata ID from a list of links
    fn extract_wikidata_id_from_links(&self, links: &[String]) -> Option<String> {
        let wikidata_url = links.iter()
            .find(|link| self.is_wikidata_url(link))?;
            
        // Extract the Wikidata ID
        let parsed_url = Url::parse(wikidata_url).ok()?;
        parsed_url.path_segments()?.nth(1).map(|s| s.to_string())
    }

    /// Check if a URL is a Wikidata entity URL
    fn is_wikidata_url(&self, url: &str) -> bool {
        (url.starts_with("https://www.wikidata.org/wiki/Q") || url.starts_with("http://www.wikidata.org/wiki/Q"))
            && url.len() > "https://www.wikidata.org/wiki/Q".len()
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdItem {
    sitelinks: WdSiteLinks,
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

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdSearchResponse {
    query: WdSearchQuery,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdSearchQuery {
    search: Vec<WdSearchResult>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
struct WdSearchResult {
    title: String,
}

#[cfg(test)]
mod tests {
    use crate::{library::Library, model::{Artist, Link}, plugins::{plugin::Plugin, plugins::Plugins}};
    use super::WikidataPlugin;

    #[test]
    fn test_existing_wikidata_url() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = WikidataPlugin::default();
        
        // Create an artist with an existing Wikidata URL link
        let mut artist = Artist {
            name: Some("Radiohead".to_string()),
            ..Default::default()
        };
        artist = library.save(&artist).unwrap();
        
        // Add a Wikidata link
        let link = Link {
            id: None,
            name: Some("Wikidata".to_string()),
            url: "https://www.wikidata.org/wiki/Q202267".to_string(),
        };
        let link = library.save(&link).unwrap();
        library.db.transaction(|t| {
            use crate::model::LinkRef;
            LinkRef::attach(t, &link, &artist.id)
        }).unwrap();
        
        // Test that the plugin finds the summary using the existing link
        let result = plugin.artist_metadata(&plugins, &library, &artist);
        match result {
            Ok(Some(metadata)) => {
                assert!(metadata.artist.summary.is_some());
                println!("Found summary: {:?}", metadata.artist.summary);
            },
            Ok(None) => println!("No metadata found (API might be unavailable)"),
            Err(e) => println!("Error occurred: {:?}", e),
        }
    }

    #[test]
    fn test_musicbrainz_fallback() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = WikidataPlugin::default();
        
        // Create an artist with only a MusicBrainz ID (no existing Wikidata link)
        let artist = Artist {
            name: Some("Radiohead".to_string()),
            musicbrainz_id: Some("a74b1b7f-71a5-4011-9441-d0b5e4122711".to_string()), // Radiohead's MusicBrainz ID
            ..Default::default()
        };
        let artist = library.save(&artist).unwrap();
        
        // Test that the plugin finds the summary using MusicBrainz ID fallback
        let result = plugin.artist_metadata(&plugins, &library, &artist);
        match result {
            Ok(Some(metadata)) => {
                assert!(metadata.artist.summary.is_some());
                // Should also include the found Wikidata URL since there was none before
                assert!(metadata.artist.wikidata_id.is_some());
                println!("Found summary via MusicBrainz fallback: {:?}", metadata.artist.summary);
                println!("Found Wikidata URL: {:?}", metadata.artist.wikidata_id);
            },
            Ok(None) => println!("No metadata found (API might be unavailable)"),
            Err(e) => println!("Error occurred: {:?}", e),
        }
    }

    #[test]
    fn test_no_sources_available() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = WikidataPlugin::default();
        
        // Create an artist with no Wikidata URL and no MusicBrainz ID
        let artist = Artist {
            name: Some("Unknown Artist".to_string()),
            ..Default::default()
        };
        let artist = library.save(&artist).unwrap();
        
        // Test that the plugin returns None when no sources are available
        let result = plugin.artist_metadata(&plugins, &library, &artist);
        match result {
            Ok(None) => println!("Correctly returned None for artist with no sources"),
            Ok(Some(_)) => panic!("Unexpected metadata found for artist with no sources"),
            Err(e) => println!("Error occurred: {:?}", e),
        }
    }

    #[test]
    fn test_preserve_existing_wikidata_id() {
        let _ = env_logger::try_init();
        let library = Library::open_memory();
        let plugins = Plugins::default();
        let plugin = WikidataPlugin::default();
        
        // Create an artist with both MusicBrainz ID and existing wikidata_id
        let artist = Artist {
            name: Some("Radiohead".to_string()),
            musicbrainz_id: Some("a74b1b7f-71a5-4011-9441-d0b5e4122711".to_string()),
            wikidata_id: Some("https://www.wikidata.org/wiki/Q202267".to_string()),
            ..Default::default()
        };
        let artist = library.save(&artist).unwrap();
        
        // Test that the plugin doesn't overwrite existing wikidata_id
        let result = plugin.artist_metadata(&plugins, &library, &artist);
        match result {
            Ok(Some(metadata)) => {
                // Should not include wikidata_id in result since it already exists
                assert!(metadata.artist.wikidata_id.is_none());
                println!("Correctly preserved existing wikidata_id");
            },
            Ok(None) => println!("No metadata found (API might be unavailable)"),
            Err(e) => println!("Error occurred: {:?}", e),
        }
    }
}





