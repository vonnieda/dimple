use crate::ui::app_window_controller::App;
use crate::ui::common::MutableStringParam;
use crate::ui::CardAdapter;
use crate::ui::CardSectionAdapter;
use crate::ui::Page;
use dimple_core::librarian;
use dimple_core::model::Artist;
use dimple_core::model::Genre;
use dimple_core::model::ModelBasics;
use dimple_core::model::Link;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::ReleaseGroupPrimaryType;
use itertools::Itertools as _;
use slint::ComponentHandle as _;
use slint::ModelRc;
use url::Url;
use crate::ui::LinkAdapter;
use crate::ui::ArtistDetailsAdapter;
use dimple_db::db::query::QuerySubscription;
use anyhow::Result;

pub struct ArtistDetailsController {
    artist_id: MutableStringParam,
    artist_subscription: QuerySubscription,
    genres_subscription: QuerySubscription,
    links_subscription: QuerySubscription,
    release_groups_subscription: QuerySubscription,
}


impl ArtistDetailsController {
    pub fn new(app: &App) -> Result<Self> {
        let artist_id = MutableStringParam::new();
        
        // Set up UI event handlers
        let app_clone = app.clone();
        let artist_id_clone = artist_id.clone();
        app.ui.upgrade_in_event_loop(move |ui| {
            let app = app_clone.clone();
            ui.global::<ArtistDetailsAdapter>().on_toggle_heart(move || {
                app.library.db.transaction(|txn| {
                    let mut artist: Artist = txn.get(&artist_id_clone.value())?.expect("artist not found");
                    artist.save = !artist.save;
                    Ok(txn.save(&artist)?)
                }).unwrap();
            });
        })?;

        // Set up artist subscription
        let sql = "SELECT * FROM Artist WHERE id = ?";
        let ui = app.ui.clone();
        let artist_subscription = app.library.db.query_subscribe(sql, (artist_id.clone(),), move |artists: Vec<Artist>| {
            if let Some(artist) = artists.first() {
                let artist = artist.clone();
                // TODO add changes to dump, or to a second debug section
                ui.upgrade_in_event_loop(move |ui| {
                    let card: CardAdapter = artist.clone().into();                
                    ui.global::<ArtistDetailsAdapter>().set_card(card);
                    ui.global::<ArtistDetailsAdapter>().set_key(artist.id.clone().unwrap_or_default().into());
                    ui.global::<ArtistDetailsAdapter>().set_save(artist.save);
                    ui.global::<ArtistDetailsAdapter>().set_summary(artist.summary.clone().unwrap_or_default().into());
                    ui.global::<ArtistDetailsAdapter>().set_disambiguation(artist.disambiguation.clone().unwrap_or_default().into());
                    ui.global::<ArtistDetailsAdapter>().set_dump(serde_json::to_string_pretty(&artist).unwrap().into());
                }).unwrap();
            }
        })?;

        // Set up genres subscription
        let sql = "
            SELECT g.* FROM Genre g
            JOIN GenreRef gr ON g.id = gr.genre_id
            WHERE gr.model_id = ?
            ORDER BY name ASC
        ";
        let ui = app.ui.clone();
        let genres_subscription = app.library.db.query_subscribe(sql, (artist_id.clone(),), move |genres: Vec<Genre>| {
            ui.upgrade_in_event_loop(move |ui| {
                let genre_links = genre_links(&genres);
                ui.global::<ArtistDetailsAdapter>().set_genres(ModelRc::from(genre_links.as_slice()));
            }).unwrap();
        })?;

        // Set up links subscription
        let sql = "
            SELECT l.* FROM Link l
            JOIN LinkRef lr ON l.id = lr.link_id
            WHERE lr.model_id = ?
            ORDER BY name ASC, url ASC
        ";
        let ui = app.ui.clone();
        let links_subscription = app.library.db.query_subscribe(sql, (artist_id.clone(),), move |links: Vec<Link>| {
            ui.upgrade_in_event_loop(move |ui| {
                let link_adapters: Vec<LinkAdapter> = links.iter().map(|link| {
                    LinkAdapter {
                        name: link.name.clone().unwrap_or_else(|| link.url.clone()).into(),
                        url: link.url.clone().into(),
                    }
                }).collect();
                ui.global::<ArtistDetailsAdapter>().set_links(ModelRc::from(link_adapters.as_slice()));
            }).unwrap();
        })?;

        // TODO for now, only showing release groups with no secondary types
        // like live. When we have filtering on the card grid we can show more
        // but for now it's too much.
        let sql = "
            SELECT ReleaseGroup.*
            FROM ReleaseGroup
            JOIN ArtistRef ON ArtistRef.model_id = ReleaseGroup.id
            LEFT JOIN ReleaseGroupSecondaryTypeRef ON ReleaseGroupSecondaryTypeRef.release_group_id = ReleaseGroup.id
            WHERE ArtistRef.artist_id = ?
            AND ReleaseGroupSecondaryTypeRef.id IS NULL
            ORDER BY ReleaseGroup.first_release_date DESC, ReleaseGroup.title ASC, ReleaseGroup.rowid
            ;
        ";
        let ui = app.ui.clone();
        let release_groups_subscription = app.library.db.query_subscribe(sql, (artist_id.clone(),), move |groups: Vec<ReleaseGroup>| {
            ui.upgrade_in_event_loop(move |ui| {
                let sections = release_group_sections(&groups);
                let adapter = ui.global::<ArtistDetailsAdapter>();
                adapter.set_releases(sections.as_slice().into());
            }).unwrap();
        })?;

        Ok(Self {
            artist_id,
            artist_subscription,
            genres_subscription,
            links_subscription,
            release_groups_subscription,
        })
    }

    pub fn set_artist(&mut self, key: String, app: &App) -> Result<()> {
        self.artist_id.set(&key);
        
        // Refresh all subscriptions
        self.artist_subscription.refresh();
        self.genres_subscription.refresh();
        self.links_subscription.refresh();
        self.release_groups_subscription.refresh();

        // Trigger metadata refresh in background
        let app_clone = app.clone();
        let key_clone = key.clone();
        std::thread::spawn(move || {
            if let Some(artist) = Artist::get(&app_clone.library, &key_clone) {
                librarian::refresh_metadata(&app_clone.library, &app_clone.plugins, &artist.into());
            }
        });

        Ok(())
    }
}

fn release_group_sections(groups: &[ReleaseGroup]) -> Vec<CardSectionAdapter> {
    let albums = groups.iter()
        .filter(|g| match g.primary_type {
            Some(ReleaseGroupPrimaryType::Album) => true,
            _ => false
        })
        .collect::<Vec<_>>();
    let singles_and_eps = groups.iter()
        .filter(|g| match g.primary_type {
            Some(ReleaseGroupPrimaryType::Single) => true,
            Some(ReleaseGroupPrimaryType::EP) => true,
            _ => false
        })
        .collect::<Vec<_>>();
    let others = groups.iter()
        .filter(|g| match g.primary_type {
            Some(ReleaseGroupPrimaryType::Album) => false,
            Some(ReleaseGroupPrimaryType::Single) => false,
            Some(ReleaseGroupPrimaryType::EP) => false,
            _ => true
        })
        .collect::<Vec<_>>();

    // 4. Map to CardSectionAdapters (Albums, Singles & EPs, Live, Other)
    let mut sections: Vec<CardSectionAdapter> = vec![];
    if !albums.is_empty() {
        let groups = albums.into_iter().sorted_by_key(|r| r.first_release_date.clone()).rev().cloned().collect::<Vec<_>>();
        sections.push(CardSectionAdapter {
            title: "Albums ⟩".into(),
            sub_title: Default::default(),
            cards: release_group_cards(groups.as_slice()).as_slice().into(),
            ..Default::default()
        });
    }
    if !singles_and_eps.is_empty() {
        let groups = singles_and_eps.into_iter().sorted_by_key(|r| r.first_release_date.clone()).rev().cloned().collect::<Vec<_>>();
        sections.push(CardSectionAdapter {
            title: "Singles & EPs ⟩".into(),
            sub_title: Default::default(),
            cards: release_group_cards(groups.as_slice()).as_slice().into(),
            ..Default::default()
        });
    }
    if !others.is_empty() {
        let groups = others.into_iter().sorted_by_key(|r| r.first_release_date.clone()).rev().cloned().collect::<Vec<_>>();
        sections.push(CardSectionAdapter {
            title: "Other Releases ⟩".into(),
            sub_title: Default::default(),
            cards: release_group_cards(groups.as_slice()).as_slice().into(),
            ..Default::default()
        });
    }

    sections
}

pub fn artist_details(url: &str, app: &App, controller: &mut ArtistDetailsController) {
    let url = Url::parse(url).unwrap();
    let key = url.path_segments().unwrap().next().unwrap().to_string();

    // Set the artist in the controller which will handle all subscriptions
    controller.set_artist(key, app).unwrap();
    
    // Navigate to the artist details page
    app.ui.upgrade_in_event_loop(move |ui| {
        ui.set_page(Page::ArtistDetails);
    }).unwrap();
}

fn genre_links(genres: &[Genre]) -> Vec<LinkAdapter> {
    genres.iter().map(|genre| {
        LinkAdapter {
            name: genre.name.clone().unwrap_or_default().into(),
            url: format!("dimple://genre/{}", genre.id.clone().unwrap_or_default()).into(),
        }
    }).collect()
}

fn release_group_cards(groups: &[ReleaseGroup]) -> Vec<CardAdapter> {
    groups.iter().cloned()
        .map(|group| {
            let card: CardAdapter = group.into();
            card
        })
        .collect()
}

