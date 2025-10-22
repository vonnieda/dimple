use std::future;
use std::rc::Rc;

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
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::plugins::plugins::Plugins;
use futures_signals::signal::Mutable;
use futures_signals::signal::SignalExt;
use itertools::Itertools as _;
use slint::ComponentHandle as _;
use slint::ModelRc;
use slint::SharedString;
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
    release_group: Mutable<Option<ReleaseGroup>>,
    genres: Mutable<Vec<Genre>>,
    artists: Mutable<Vec<Artist>>,
    links: Mutable<Vec<Link>>,
    releases: Mutable<Vec<Release>>,
}

impl ReleaseGroupDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let mut controller = ReleaseGroupDetailsController {
            library: app.library.clone(),
            ui: app.ui.clone(),
            plugins: app.plugins.clone(),
            release_group_id: Default::default(),
            release_group: Default::default(),
            genres: Default::default(),
            artists: Default::default(),
            links: Default::default(),
            releases: Default::default(),
        };
        controller.init()?;
        Ok(controller)
    }

    fn init(&mut self) -> anyhow::Result<()> {
        let release_group_id_query_param = MutableStringParam::default();

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

        // Get the release group artists
        let artists_clone = self.artists.clone();
        let sql = "
            SELECT Artist.* FROM Artist
            JOIN ArtistRef ON (Artist.id = ArtistRef.artist_id)
            WHERE ArtistRef.model_id = ?1
            ORDER BY ArtistRef.rowid ASC
        ";
        let artists_query = self.library.db.query_subscribe(
            sql,
            (release_group_id_query_param.clone(),),
            move |artists: Vec<Artist>| {
                artists_clone.set(artists);
            }
        )?;

        // Get the release group genres
        let genres_clone = self.genres.clone();
        let sql = "
            SELECT Genre.* FROM Genre
            JOIN GenreRef ON (Genre.id = GenreRef.genre_id)
            WHERE GenreRef.model_id = ?1
            ORDER BY Genre.name ASC
        ";
        let genres_query = self.library.db.query_subscribe(
            sql,
            (release_group_id_query_param.clone(),),
            move |genres: Vec<Genre>| {
                genres_clone.set(genres);
            }
        )?;

        // Get the release group links
        let links_clone = self.links.clone();
        let sql = "
            SELECT Link.* FROM Link
            JOIN LinkRef ON (Link.id = LinkRef.link_id)
            WHERE LinkRef.model_id = ?1
            ORDER BY Link.name ASC, Link.url ASC
        ";
        let links_query = self.library.db.query_subscribe(
            sql,
            (release_group_id_query_param.clone(),),
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

        // When the release_group_id changes set the query parameter and refresh
        // the query.
        let release_group_id_query_param_clone = release_group_id_query_param.clone();
        spawn(self.release_group_id.signal_cloned().for_each(move |release_group_id| {
            if let Some(release_group_id) = release_group_id {
                release_group_id_query_param_clone.set(&release_group_id);
                release_group_query.refresh();
            }
            future::ready(())
        }));

        // When the release group changes, refresh queries and push data to UI
        let library_clone = self.library.clone();
        let plugins_clone = self.plugins.clone();
        let ui_clone = self.ui.clone();
        spawn(self.release_group.signal_cloned().for_each(move |release_group| {
            if let Some(release_group) = release_group {
                artists_query.refresh();
                genres_query.refresh();
                links_query.refresh();
                releases_query.refresh();

                let release_group_clone = release_group.clone();
                let secondary_types = release_group_clone.secondary_types(&library_clone).unwrap();
                ui_clone.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = release_group_clone.clone().into();
                    ui.global::<ReleaseGroupDetailsAdapter>().set_card(card);
                    ui.global::<ReleaseGroupDetailsAdapter>().set_key(release_group_clone.id.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_save(release_group_clone.save);
                    ui.global::<ReleaseGroupDetailsAdapter>().set_first_release_date(release_group_clone.first_release_date.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_primary_type(
                        release_group_clone.primary_type.clone().map(|t| format!("{:?}", t)).unwrap_or_default().into()
                    );
                    let secondary_types: Vec<SharedString> = secondary_types.iter().map(|t| t.to_string().into()).collect_vec();
                    ui.global::<ReleaseGroupDetailsAdapter>().set_secondary_types(secondary_types.as_slice().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_summary(release_group_clone.summary.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_disambiguation(release_group_clone.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ReleaseGroupDetailsAdapter>().set_dump(serde_json::to_string_pretty(&release_group_clone).unwrap().into());
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

        let ui_clone = self.ui.clone();
        spawn(self.releases.signal_cloned().for_each(move |releases| {
            ui_clone.upgrade_in_event_loop(move |ui| {
                let cards = release_cards(&releases);
                ui.global::<ReleaseGroupDetailsAdapter>().set_releases(ModelRc::from(cards.as_slice()));
            }).unwrap();
            future::ready(())
        }));

        // Set up UI event handlers
        let release_group_clone = self.release_group.clone();
        let library_clone = self.library.clone();
        self.ui.upgrade_in_event_loop(move |ui| {
            ui.global::<ReleaseGroupDetailsAdapter>().on_toggle_heart(move || {
                library_clone.db.transaction(|txn| {
                    if let Some(release_group) = release_group_clone.get_cloned() {
                        let mut release_group_clone: ReleaseGroup = txn.get(&release_group.id.unwrap())?.unwrap();
                        release_group_clone.save = !release_group.save;
                        let _ = txn.save(&release_group_clone)?;
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
            self.release_group_id.set(Some(release_group_id));
            self.ui.upgrade_in_event_loop(move |ui| {
                ui.set_page(Page::ReleaseGroupDetails);
            }).unwrap();
        }
    }
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
