use anyhow::Result;
use dimple_core::model::ReleaseGroup;
use crate::ui::app_window_controller::App;
use crate::ui::CardAdapter;
use crate::ui::CardSectionAdapter;
use crate::ui::LinkAdapter;
use crate::ui::GenreDetailsAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::common::MutableStringParam;
use crate::ui::Page;
use dimple_core::library::Library;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Release;
use dimple_db::db::query::QuerySubscription;
use slint::ComponentHandle as _;
use slint::ModelRc;

pub struct GenreDetailsController {
    app: App,
    genre_id: MutableStringParam,
    genre_subscription: QuerySubscription,
    releases_subscription: QuerySubscription,
    artists_subscription: QuerySubscription,
    links_subscription: QuerySubscription,
}

impl GenreDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let genre_id = MutableStringParam::new();
        
        // Set up UI event handlers
        let app_clone = app.clone();
        let genre_id_clone = genre_id.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_clone.clone();
            ui.global::<GenreDetailsAdapter>().on_toggle_heart(move || {
                app.library.db.transaction(|txn| {
                    let mut genre: Genre = txn.get(&genre_id_clone.value())?.expect("genre not found");
                    genre.save = !genre.save;
                    Ok(txn.save(&genre)?)
                }).unwrap();
            });
        })?;
        
        // Set up genre subscription
        let sql = "SELECT * FROM Genre WHERE id = ?";
        let ui = app.ui.clone();
        let genre_subscription = app.library.db.query_subscribe(sql, (genre_id.clone(),), move |genres: Vec<Genre>| {
            if let Some(genre) = genres.first() {
                let genre = genre.clone();
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = genre.clone().into();
                    ui.global::<GenreDetailsAdapter>().set_card(card);
                    ui.global::<GenreDetailsAdapter>().set_key(genre.id.clone().unwrap_or_default().into());
                    ui.global::<GenreDetailsAdapter>().set_save(genre.save);
                    ui.global::<GenreDetailsAdapter>().set_summary(genre.summary.clone().unwrap_or_default().into());
                    ui.global::<GenreDetailsAdapter>().set_disambiguation(genre.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<GenreDetailsAdapter>().set_dump(serde_json::to_string_pretty(&genre).unwrap().into());
                }).unwrap();
            }
        })?;

        // Set up links subscription
        let sql = "
            SELECT l.* FROM LinkRef lr 
            JOIN Link l ON (l.id = lr.link_id) 
            WHERE lr.model_id = ?1
            ORDER BY name ASC, url ASC
        ";
        let ui = app.ui.clone();
        let links_subscription = app.library.db.query_subscribe(sql, (genre_id.clone(),), move |links: Vec<dimple_core::model::Link>| {
            ui.upgrade_in_event_loop(move |ui| {
                let link_adapters: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                        url: link.url.clone().into(),
                    }
                }).collect();
                ui.global::<GenreDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        // Set up releases subscription
        let sql = "
            SELECT ReleaseGroup.* FROM ReleaseGroup
            LEFT JOIN GenreRef ON (GenreRef.model_id = ReleaseGroup.id)
            WHERE GenreRef.genre_id = ?1
            ORDER BY title ASC
        ";
        let ui = app.ui.clone();
        let library = app.library.clone();
        let releases_subscription = app.library.db.query_subscribe(sql, (genre_id.clone(),), move |releases: Vec<ReleaseGroup>| {
            let library = library.clone();
            ui.upgrade_in_event_loop(move |ui| {
                let mut sections: Vec<CardSectionAdapter> = vec![];
                
                if !releases.is_empty() {
                    let cards = release_group_cards(&releases);
                    sections.push(CardSectionAdapter {
                        title: "Releases".into(),
                        sub_title: Default::default(),
                        cards: cards.as_slice().into(),
                        ..Default::default()
                    });
                }
                
                ui.global::<GenreDetailsAdapter>().set_sections(sections.as_slice().into());
            }).unwrap();
        })?;

        // Set up artists subscription
        let sql = "
            SELECT Artist.* FROM Artist
            LEFT JOIN GenreRef ON (GenreRef.model_id = Artist.id)
            WHERE GenreRef.genre_id = ?1
            ORDER BY name ASC
        ";
        let ui = app.ui.clone();
        let artists_subscription = app.library.db.query_subscribe(sql, (genre_id.clone(),), move |artists: Vec<Artist>| {
            ui.upgrade_in_event_loop(move |ui| {
                let mut sections: Vec<CardSectionAdapter> = vec![];
                
                if !artists.is_empty() {
                    let cards = artist_cards(&artists);
                    sections.push(CardSectionAdapter {
                        title: "Artists".into(),
                        sub_title: Default::default(),
                        cards: cards.as_slice().into(),
                        ..Default::default()
                    });
                }
                
                ui.global::<GenreDetailsAdapter>().set_sections(sections.as_slice().into());
            }).unwrap();
        })?;

        Ok(Self {
            app: app.clone(),
            genre_id,
            genre_subscription,
            releases_subscription,
            artists_subscription,
            links_subscription,
        })
    }

    pub fn set_genre_id(&self, id: &str) {
        self.genre_id.set(id);
        self.genre_subscription.refresh();
        self.links_subscription.refresh();
        self.artists_subscription.refresh();
        self.releases_subscription.refresh();
    }

    pub fn navigate(&self, url: &str) {
        let url = url::Url::parse(&url).unwrap();
        let id = url.path_segments().unwrap().next().unwrap().to_string();
        self.set_genre_id(&id);
        self.app.ui.upgrade_in_event_loop(|ui| ui.set_page(Page::GenreDetails)).unwrap();
    }
}

fn release_group_cards(groups: &[ReleaseGroup]) -> Vec<CardAdapter> {
    groups.iter().cloned()
        .map(|group| {
            let card: CardAdapter = group.into();
            card
        })
        .collect()
}

fn release_cards(releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned()
        .map(|release| release_card(&release, &release.artist(library).unwrap_or_default()))
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
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
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.id.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}

fn artist_cards(artists: &[Artist]) -> Vec<CardAdapter> {
    artists.iter().cloned()
        .map(|artist| artist_card(&artist))
        .collect()
}

fn artist_card(artist: &Artist) -> CardAdapter {
    let artist = artist.clone();
    CardAdapter {
        key: artist.id.clone().unwrap_or_default().into(),        
        image: ImageLinkAdapter {
            image: Default::default(),
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        sub_title: LinkAdapter {
            name: artist.disambiguation.unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.id.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}