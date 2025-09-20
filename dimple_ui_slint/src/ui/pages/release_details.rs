use std::future;
use std::rc::Rc;
use std::time::Duration;

use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::AppWindow;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::Page;
use crate::ui::ReleaseDetailsAdapter;
use dimple_core::librarian;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::Link;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use dimple_core::plugins::plugins::Plugins;
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
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
pub struct ReleaseDetailsController {
    library: Library,
    ui: Weak<AppWindow>,
    plugins: Plugins,

    release_id: Mutable<Option<String>>,
    release: Mutable<Option<Release>>,
    release_group: Mutable<Option<ReleaseGroup>>,
    genres: Mutable<Vec<Genre>>,
    artists: Mutable<Vec<Artist>>,
    links: Mutable<Vec<Link>>,
    other_versions: Mutable<Vec<Release>>,
    tracks: Mutable<Vec<Track>>,

    // release_query: Option<QuerySubscription>,
}

impl ReleaseDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let mut controller = ReleaseDetailsController {
            library: app.library.clone(),
            ui: app.ui.clone(),
            plugins: app.plugins.clone(),
            release_group: Default::default(),
            genres: Default::default(),
            artists: Default::default(),
            links: Default::default(),
            release_id: Default::default(),
            release: Default::default(),
            tracks: Default::default(),
            other_versions: Default::default(),
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
        let release_id_query_param = MutableStringParam::default();

        let release_clone = self.release.clone();
        let sql = "
            SELECT * 
            FROM Release 
            WHERE id = ?
        ";
        let release_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |releases: Vec<Release>| {
                release_clone.set(releases.first().cloned());
            }
        )?;

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

        let links_clone = self.links.clone();
        let sql = "
            SELECT l.* FROM LinkRef lr 
            JOIN Link l ON (l.id = lr.link_id) 
            WHERE lr.model_id = ?1
        ";
        let links_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |links: Vec<Link>| {
                links_clone.set(links);
            }
        )?;

        let other_versions_clone = self.other_versions.clone();
        let sql = "
            SELECT Release.* 
            FROM Release 
            WHERE Release.release_group_id = (SELECT release_group_id FROM Release WHERE id = ?1)
            AND Release.id != ?1
            ORDER BY Release.date ASC NULLS LAST, Release.title ASC, Release.id ASC
        ";
        let other_versions_query = self.library.db.query_subscribe(
            sql, 
            (release_id_query_param.clone(),), 
            move |releases: Vec<Release>| {
                other_versions_clone.set(releases);
            }
        )?;

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

        let release_id_query_param_clone = release_id_query_param.clone();
        spawn(self.release_id.signal_cloned().for_each(move |release_id| {
            if let Some(release_id) = release_id {
                release_id_query_param_clone.set(&release_id);
                release_query.refresh();
            }
            future::ready(())
        }));

        let library_clone = self.library.clone();
        let plugins_clone = self.plugins.clone();
        let ui_clone = self.ui.clone();
        spawn(self.release.signal_cloned().for_each(move |release| {
            if let Some(release) = release {
                artists_query.refresh();
                genres_query.refresh();
                links_query.refresh();
                tracks_query.refresh();
                other_versions_query.refresh();

                let release_clone = release.clone();
                ui_clone.upgrade_in_event_loop(move |ui| {
                    let release = release_clone;
                    let card: CardAdapter = release.clone().into();
                    ui.global::<ReleaseDetailsAdapter>().set_card(card);
                    ui.global::<ReleaseDetailsAdapter>().set_key(release.id.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_save(release.save);
                    ui.global::<ReleaseDetailsAdapter>().set_summary(release.summary.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_disambiguation(release.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ReleaseDetailsAdapter>().set_dump(serde_json::to_string_pretty(&release).unwrap().into());
                }).unwrap();

                let library_clone = library_clone.clone();
                let plugins_clone = plugins_clone.clone();
                std::thread::spawn(move || {
                    librarian::refresh_metadata(&library_clone, &plugins_clone, &release.into());
                });
            }
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.artists.signal_cloned().for_each(move |artists| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let artist_links = artist_links(&artists);
                ui.global::<ReleaseDetailsAdapter>().set_artists(ModelRc::from(artist_links.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.genres.signal_cloned().for_each(move |genres| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let genre_links = genre_links(&genres);
                ui.global::<ReleaseDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.links.signal_cloned().for_each(move |links| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let link_links = link_links(&links);
                ui.global::<ReleaseDetailsAdapter>().set_links(ModelRc::from(link_links.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        let ui_clone = self.ui.clone();
        spawn(self.other_versions.signal_cloned().for_each(move |releases| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let cards = release_version_cards(&releases);
                ui.global::<ReleaseDetailsAdapter>().set_other_versions(ModelRc::from(cards.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        // When the tracks change push them to the UI. 
        let ui_clone = self.ui.clone();
        let library_clone = self.library.clone();
        spawn(self.tracks.signal_cloned().for_each(move |tracks| {
            let library_clone = library_clone.clone();
            ui_clone.upgrade_in_event_loop(move |ui| {
                ui.global::<ReleaseDetailsAdapter>().set_track_items(track_items(&library_clone, &tracks));
                ui.global::<ReleaseDetailsAdapter>().set_track_keys(track_keys(&tracks));
            }).unwrap();
            future::ready(())
        }));

        // Set up UI event handlers
        let release_clone = self.release.clone();
        let library_clone = self.library.clone();
        self.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<ReleaseDetailsAdapter>().on_toggle_heart(move || {
                library_clone.db.transaction(|txn| {
                    if let Some(release) = release_clone.get_cloned() {
                        let mut release: Release = txn.get(&release.id.unwrap())?.unwrap();
                        release.save = !release.save;
                        let _ = txn.save(&release)?;
                    }
                    Ok(())
                }).unwrap();
            });
        })?;

        Ok(())
    }

    pub fn navigate(&self, url: &str) {
        let url = Url::parse(url).unwrap();
        if url.as_str().starts_with("dimple://releasegroup/") {
            let release_group_id = url.path_segments().unwrap().next().unwrap().to_string();
            if let Some(release_group) = ReleaseGroup::get(&self.library, &release_group_id) {
                if let Some(release) = release_group.releases(&self.library).first() {
                    self.release_id.set(release.id.clone());
                    self.ui.upgrade_in_event_loop(move |ui| {
                        ui.set_page(Page::ReleaseDetails);
                    }).unwrap();
                }
            }
        }
        else if url.as_str().starts_with("dimple://release/") {
            let release_id = url.path_segments().unwrap().next().unwrap().to_string();
            self.release_id.set(Some(release_id));
            self.ui.upgrade_in_event_loop(move |ui| {
                ui.set_page(Page::ReleaseDetails);
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
        row.push(track.position.unwrap_or_default().to_string().as_str().into()); // Track #
        row.push(track.title.clone().unwrap_or_default().as_str().into()); // Title
        // TODO artists
        row.push(track.artist_name(library).unwrap_or_default().as_str().into()); // Artist
        row.push(length.unwrap_or_default().as_str().into()); // Length
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

fn release_version_cards(releases: &[Release]) -> Vec<CardAdapter> {
    releases.iter().cloned()
        .map(|release| release_version_card(&release))
        .collect()
}

fn release_version_card(release: &Release) -> CardAdapter {
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
