use std::future;
use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::Page;
use crate::ui::ReleaseGroupDetailsAdapter;
use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use dimple_core::model::TrackSource;
use dimple_core::plugins::plugins::Plugins;
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use itertools::Itertools as _;
use slint::ComponentHandle as _;
use slint::ModelRc;
use slint::SharedString;
use slint::StandardListViewItem;
use slint::VecModel;
use slint::Weak;
use tokio::spawn;
use url::Url;
use crate::ui::LinkAdapter;
use anyhow::Result;

#[derive(Clone)]
pub struct ReleaseGroupDetailsController {
    library: Library,
    ui: Weak<AppWindow>,
    plugins: Plugins,

    release_group_id: Mutable<Option<String>>,
    release_id: Mutable<Option<String>>,

    release_group: Mutable<Option<ReleaseGroup>>,
    genres: Mutable<Vec<Genre>>,
    artists: Mutable<Vec<Artist>>,
    links: Mutable<Vec<Link>>,
    releases: Mutable<Vec<Release>>,
    release: Mutable<Option<Release>>,
    tracks: Mutable<Vec<Track>>,
}

impl ReleaseGroupDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let mut controller = ReleaseGroupDetailsController {
            library: app.library.clone(),
            ui: app.ui.clone(),
            plugins: app.plugins.clone(),
            release_group: Default::default(),
            genres: Default::default(),
            artists: Default::default(),
            links: Default::default(),
            release_id: Default::default(),
            release_group_id: Default::default(),
            release: Default::default(),
            tracks: Default::default(),
            releases: Default::default(),
        };
        controller.init()?;
        Ok(controller)
    }

    // digraph G {
    //     release_group_id -> release_group;
    //     release_group -> {genres artists links releases};
    //     {releases release_id}-> release;
    //     release -> tracks;
    // }
    // https://dreampuf.github.io/GraphvizOnline/?engine=dot#digraph%20G%20%7B%0A%20%20%20%20release_group_id%20-%3E%20release_group%3B%0A%20%20%20%20release_group%20-%3E%20%7Bgenres%20artists%20links%20releases%7D%3B%0A%20%20%20%20%7Breleases%20release_id%7D-%3E%20release%3B%0A%20%20%20%20release%20-%3E%20tracks%3B%0A%7D
    fn init(&mut self) -> anyhow::Result<()> {        
        let release_group_id_query_param = MutableStringParam::default();
        let release_id_query_param = MutableStringParam::default();

        // Gets the current release group using the release_group_id
        let release_group_clone = self.release_group.clone();
        let sql = "
            SELECT * 
            FROM ReleaseGroup
            WHERE id = ?
        ";
        let release_group_query = self.library.db.query_subscribe(
            sql, 
            (release_group_id_query_param.clone(),), 
            move |release_groups: Vec<ReleaseGroup>| {
                release_group_clone.set(release_groups.first().cloned());
            }
        )?;

        // Get the release artists
        let artists_clone = self.artists.clone();
        let sql = "
            SELECT Artist.* FROM Artist 
            JOIN ArtistRef ON (Artist.id = ArtistRef.artist_id) 
            WHERE ArtistRef.model_id = ?1
            ORDER BY ArtistRef.rowid ASC
        ";
        let artists_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |artists: Vec<Artist>| {
                artists_clone.set(artists);
            }
        )?;

        // Get the release genres
        let genres_clone = self.genres.clone();
        let sql = "
            SELECT Genre.* FROM Genre 
            JOIN GenreRef ON (Genre.id = GenreRef.genre_id) 
            WHERE GenreRef.model_id = ?1
            ORDER BY Genre.name ASC
        ";
        let genres_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |genres: Vec<Genre>| {
                genres_clone.set(genres);
            }
        )?;

        // Get the release links
        let links_clone = self.links.clone();
        let sql = "
            SELECT Link.* FROM Link 
            JOIN LinkRef ON (Link.id = LinkRef.link_id) 
            WHERE LinkRef.model_id = ?1
            ORDER BY Link.name ASC, Link.url ASC
        ";
        let links_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |links: Vec<Link>| {
                links_clone.set(links);
            }
        )?;

        // Get the release group releases
        let releases_clone = self.releases.clone();
        let sql = "
            SELECT Release.* 
            FROM Release 
            WHERE Release.release_group_id = ?1
            ORDER BY Release.date ASC NULLS LAST, Release.title ASC, Release.id ASC
        ";
        let releases_query = self.library.db.query_subscribe(
            sql, 
            (release_group_id_query_param.clone(),), 
            move |releases: Vec<Release>| {
                releases_clone.set(releases);
            }
        )?;

        // Get the current release using the release_id
        let release_clone = self.release.clone();
        let sql = "
            SELECT * 
            FROM Release
            WHERE id = ?1
        ";
        let release_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |releases: Vec<Release>| {
                release_clone.set(releases.first().cloned());
            }
        )?;

        // Get the release tracks
        let tracks_clone = self.tracks.clone();
        let sql = "
            SELECT Track.* FROM Track
            WHERE Track.release_id = ?1
            ORDER BY media_position ASC, position ASC
        ";
        let tracks_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |tracks: Vec<Track>| {
                tracks_clone.set(tracks);
            }
        )?;

        // When the release_group_id changes set the query parameter and refresh
        // the query.
        // Q: Why not just do the query here?
        // A: By using the query_subscribe mechanism we also get free updates
        // from the database whenever data affecting the query changes, making
        // the UI reactive to database updates.
        let release_group_id_query_param_clone = release_group_id_query_param.clone();
        spawn(self.release_group_id.signal_cloned().for_each(move |release_group_id| {
            if let Some(release_group_id) = release_group_id {
                release_group_id_query_param_clone.set(&release_group_id);
                release_group_query.refresh();
            }
            future::ready(())
        }));

        // When the release group changes, refresh the artists, links, genres,
        // and releases queries, push the release group details to the UI, and
        // start a task to refresh the plugin metadata for the release group.
        let library_clone = self.library.clone();
        let plugins_clone = self.plugins.clone();
        let ui_clone = self.ui.clone();
        spawn(self.release_group.signal_cloned().for_each(move |release_group| {
            if let Some(release_group) = release_group {
                releases_query.refresh();

                let release_group_clone = release_group.clone();
                ui_clone.upgrade_in_event_loop(move |ui| {
                    let release_group = release_group_clone;
                    // TODO card, and basically all else, is release with
                    // fallback to release group, I think.
                    // let card: CardAdapter = release_group.clone().into();
                    // ui.global::<ReleaseGroupDetailsAdapter>().set_card(card);
                    // ui.global::<ReleaseGroupDetailsAdapter>().set_key(release_group.id.clone().unwrap_or_default().into());
                    // ui.global::<ReleaseGroupDetailsAdapter>().set_save(release_group.save);
                    // ui.global::<ReleaseGroupDetailsAdapter>().set_summary(release_group.summary.clone().unwrap_or_default().into());
                    // ui.global::<ReleaseGroupDetailsAdapter>().set_disambiguation(release_group.disambiguation.clone().unwrap_or_default().into());
                    // ui.global::<ReleaseGroupDetailsAdapter>().set_dump(serde_json::to_string_pretty(&release_group).unwrap().into());
                }).unwrap();

                let library_clone = library_clone.clone();
                let plugins_clone = plugins_clone.clone();
                std::thread::spawn(move || {
                    librarian::refresh_metadata(&library_clone, &plugins_clone, &release_group.into());
                });
            }
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.artists.signal_cloned().for_each(move |artists| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let artist_links = artist_links(&artists);
                ui.global::<ReleaseGroupDetailsAdapter>().set_artists(ModelRc::from(artist_links.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.genres.signal_cloned().for_each(move |genres| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let genre_links = genre_links(&genres);
                ui.global::<ReleaseGroupDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.links.signal_cloned().for_each(move |links| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let link_links = link_links(&links);
                ui.global::<ReleaseGroupDetailsAdapter>().set_links(ModelRc::from(link_links.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        // When releases changes, validate that the release_id is still contained
        // within the list, otherwise set it to a new, valid, default choice.
        let ui_clone = self.ui.clone();
        let release_id_clone = self.release_id.clone();
        spawn(self.releases.signal_cloned().for_each(move |releases| {
            if releases.iter().find(|r| r.id == release_id_clone.get_cloned()).is_none() {
                if let Some(release) = releases.first() {
                    release_id_clone.set(release.id.clone());
                }
                else {
                    release_id_clone.set(None);
                }
            }
            ui_clone.upgrade_in_event_loop(move |ui| {
                let cards = release_cards(&releases);
                ui.global::<ReleaseGroupDetailsAdapter>().set_releases(ModelRc::from(cards.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        // When release_id changes, update the release_id_query_param and refresh
        // the release_query. 
        // let release_id_query_param_clone = release_id_query_param.clone();
        spawn(self.release_id.signal_cloned().for_each(move |release_id| {
            if let Some(release_id) = release_id {
                release_id_query_param.set(&release_id);
                release_query.refresh();
            }
            future::ready(())
        }));

        // When the selected release changes, refresh the tracks query, and
        // and schedule a metadata refresh for the release.
        let ui_clone = self.ui.clone();
        let library_clone = self.library.clone();
        let plugins_clone = self.plugins.clone();
        spawn(self.release.signal_cloned().for_each(move |release| {
            artists_query.refresh();
            genres_query.refresh();
            links_query.refresh();
            tracks_query.refresh();
            if let Some(release) = release {
                let release_clone = release.clone();
                ui_clone.upgrade_in_event_loop(move |ui| {
                    let release = release_clone;
                    let card: CardAdapter = release.clone().into();
                    ui.global::<ReleaseGroupDetailsAdapter>().set_card(card);
                    ui.global::<ReleaseGroupDetailsAdapter>().set_key(release.id.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_save(release.save);
                    ui.global::<ReleaseGroupDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_dump(serde_json::to_string_pretty(&release).unwrap().into());
                }).unwrap();

                let library_clone = library_clone.clone();
                let plugins_clone = plugins_clone.clone();
                std::thread::spawn(move || {
                    librarian::refresh_metadata(&library_clone, &plugins_clone, &release.into());
                });
            }
            future::ready(())
        }));

        // When the tracks change push them to the UI. 
        let ui_clone = self.ui.clone();
        let library_clone = self.library.clone();
        spawn(self.tracks.signal_cloned().for_each(move |tracks| {
            let library_clone = library_clone.clone();
            ui_clone.upgrade_in_event_loop(move |ui| {
                ui.global::<ReleaseGroupDetailsAdapter>().set_track_items(track_items(&library_clone, &tracks));
                ui.global::<ReleaseGroupDetailsAdapter>().set_track_keys(track_keys(&tracks));
            }).unwrap();
            future::ready(())
        }));

        // Set up UI event handlers
        let release_clone = self.release.clone();
        let library_clone = self.library.clone();
        self.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<ReleaseGroupDetailsAdapter>().on_toggle_heart(move || {
                library_clone.db.transaction(|txn| {
                    if let Some(release) = release_clone.get_cloned() {
                        let mut release_clone: Release = txn.get(&release.id.unwrap())?.unwrap();
                        release_clone.save = !release.save;
                        let _ = txn.save(&release_clone)?;
                    }
                    Ok(())
                }).unwrap();
            });
        })?;

        Ok(())
    }

    // TODO this needs to be better - it's super slow when navigating to a
    // release. We should not need to get the release groups first. 
    // I'm thiiiiis close to calling it album.
    pub fn navigate(&self, url: &str) {
        let url = Url::parse(url).unwrap();
        if url.as_str().starts_with("dimple://releasegroup/") {
            let release_group_id = url.path_segments().unwrap().next().unwrap().to_string();
            self.release_group_id.set(Some(release_group_id));
            self.ui.upgrade_in_event_loop(move |ui| {
                ui.set_page(Page::ReleaseGroupDetails);
            }).unwrap();
        }
        else if url.as_str().starts_with("dimple://release/") {
            let release_id = url.path_segments().unwrap().next().unwrap().to_string();
            if let Some(release) = Release::get(&self.library, &release_id) {
                self.release_group_id.set(release.release_group_id.clone());
            }
            self.release_id.set(Some(release_id));
            self.ui.upgrade_in_event_loop(move |ui| {
                ui.set_page(Page::ReleaseGroupDetails);
            }).unwrap();
        }
    }

}

fn track_items(library: &Library, tracks: &[Track]) -> ModelRc<ModelRc<StandardListViewItem>> {
    let track_items: Rc<VecModel<ModelRc<StandardListViewItem>>> = Rc::new(VecModel::default());
    for track in tracks {
        let track = track.clone();
        let row = Rc::new(VecModel::default());
        let length = track.length_ms
            .map(|ms| Duration::from_millis(ms))
            .map(format_length);

        let source_count = library.query::<TrackSource, _>("
            SELECT * FROM TrackSource WHERE track_id = ?1
        ", (&track.id,)).len();

        row.push(track.position.unwrap_or_default().to_string().as_str().into()); // Track #
        row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
        let artists = track.artists(library);
        let artists_s = artists.iter().map(|a| a.name.clone().unwrap_or_default()).join(", ");
        row.push(artists_s.as_str().into()); // Artist
        row.push(length.unwrap_or_default().as_str().into()); // Length
        row.push(source_count.to_string().as_str().into()); // Sources
        track_items.push(row.into());
    }
    track_items.into()
}

fn track_keys(tracks: &[Track]) -> ModelRc<SharedString> {
    let keys: Vec<_> = tracks.iter()
        .map(|track| track.id.clone().unwrap())
        .map(SharedString::from)
        .collect();
    keys.as_slice().into()
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn artist_links(artists: &[Artist]) -> Vec<LinkAdapter> {
    artists.iter().map(|artist| {
        LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn link_links(links: &[Link]) -> Vec<LinkAdapter> {
    links.iter().map(|link| {
        LinkAdapter {
            name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
            url: link.url.clone().into(),
        }
    }).collect()
}

fn format_length(length: Duration) -> String {
    let minutes = length.as_secs() / 60;
    let seconds = length.as_secs() % 60;
    format!("{minutes}:{seconds:02}")
}

fn release_cards(releases: &[Release]) -> Vec<CardAdapter> {
    releases.iter().cloned()
        .map(|release| release_card(&release))
        .collect()
}

fn release_card(release: &Release) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        key: release.id.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: format!("{} • {}", 
                release.date.clone().unwrap_or_default(), 
                release.country.clone().unwrap_or_default()).into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: release.disambiguation.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}
