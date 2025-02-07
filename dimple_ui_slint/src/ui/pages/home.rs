use dimple_core::library::Library;
use dimple_core::model::{Artist, Release};
use slint::{ComponentHandle as _, ModelRc};

use crate::ui::app_window_controller::App;
use crate::ui::images::ImageMangler;
use crate::ui::{AppWindow, CardAdapter, HomeAdapter, HomeSectionAdapter, ImageLinkAdapter, LinkAdapter, Page};

pub fn home_init(app: &App) {
}

// For You (From your listens.)
// Recent Favorites
// New Releases
// Popular (With MusicBrainz listeners.)
// Old Favorites
pub fn home(app: &App) {    
    let app = app.clone();
    std::thread::spawn(move || {
        let ui = app.ui.clone();
        ui.upgrade_in_event_loop(move |ui| {
            update_model(&app, &ui);
            ui.set_page(Page::Home);
        })
        .unwrap();
    });
}

fn update_model(app: &App, ui: &AppWindow) {
    let mut sections: Vec<HomeSectionAdapter> = vec![];

    sections.push(HomeSectionAdapter {
        title: "New Releases".into(),
        sub_title: Default::default(),
        cards: new_releases(&app),
    });
    sections.push(HomeSectionAdapter {
        title: "Made For You".into(),
        sub_title: Default::default(),
        cards: made_for_you(&app),
    });
    sections.push(HomeSectionAdapter {
        title: "Jump Back In".into(),
        sub_title: Default::default(),
        cards: jump_back_in(&app),
    });
    sections.push(HomeSectionAdapter {
        title: "Popular".into(),
        sub_title: "with musicbrainz listeners".into(),
        cards: new_releases(&app),
    });
    sections.push(HomeSectionAdapter {
        title: "All Time Faves".into(),
        sub_title: Default::default(),
        cards: all_time_faves(&app),
    });

    let adapter = ui.global::<HomeAdapter>();
    adapter.set_sections(sections.as_slice().into());
}

fn new_releases(app: &App) -> ModelRc<CardAdapter> {
    let releases: Vec<Release> = app.library.query("
        SELECT * FROM Release ORDER BY date DESC LIMIT 10 
    ", ());
    let images = app.images.clone();
    let cards: Vec<CardAdapter> = release_cards(&images, &releases, &app.library);
    cards.as_slice().into()
}

fn made_for_you(app: &App) -> ModelRc<CardAdapter> {
    let releases: Vec<Release> = app.library.query("
        SELECT * FROM Release ORDER BY random() LIMIT 10 
    ", ());
    let images = app.images.clone();
    let cards: Vec<CardAdapter> = release_cards(&images, &releases, &app.library);
    cards.as_slice().into()
}

fn jump_back_in(app: &App) -> ModelRc<CardAdapter> {
    let releases: Vec<Release> = app.library.query("
        SELECT Release.* 
        FROM Release 
        JOIN ArtistRef ON (ArtistRef.model_key = Release.key)
        JOIN Artist ON (Artist.key = ArtistRef.artist_key)
        JOIN 
            (SELECT artist,album,count(title) AS cnt 
                FROM Event 
                WHERE (event_type = 'track_played' OR event_type = 'track_restarted') 
                GROUP BY artist,album) AS Ranks 
            ON (Release.title = Ranks.album AND Artist.name = Ranks.artist)
        WHERE cnt > 3
        ORDER BY random() LIMIT 10;
        ", ());
    let images = app.images.clone();
    let cards: Vec<CardAdapter> = release_cards(&images, &releases, &app.library);
    cards.as_slice().into()
}

fn all_time_faves(app: &App) -> ModelRc<CardAdapter> {
    // SELECT artist,album FROM Event GROUP BY artist,album ORDER BY COUNT(*) DESC LIMIT 10
    let releases: Vec<Release> = app.library.query("
        SELECT Release.* 
        FROM Release 
        JOIN ArtistRef ON (ArtistRef.model_key = Release.key)
        JOIN Artist ON (Artist.key = ArtistRef.artist_key)
        JOIN 
            (SELECT artist,album,count(title) AS cnt 
                FROM Event 
                WHERE (event_type = 'track_played' OR event_type = 'track_restarted') 
                GROUP BY artist,album) AS Ranks 
            ON (Release.title = Ranks.album AND Artist.name = Ranks.artist)
        ORDER BY Ranks.cnt DESC LIMIT 10;
    ", ());
    let images = app.images.clone();
    let cards: Vec<CardAdapter> = release_cards(&images, &releases, &app.library);
    cards.as_slice().into()
}


// select row_number() over (order by count(artist) desc),count(artist),artist from event group by artist limit 10;

fn release_cards(images: &ImageMangler, releases: &[Release], library: &Library) -> Vec<CardAdapter> {
    releases.iter().cloned().enumerate()
        .map(|(index, release)| {
            let mut card: CardAdapter = release_card(&release, &release.artist(library).unwrap_or_default());
            card.image.image = images.lazy_get(release.clone(), 200, 200, move |ui, image| {
                // let adapter = ui.global::<GenreDetailsAdapter>();
                // let mut card = adapter.get_releases().row_data(index).unwrap();
                // card.image.image = image;
                // adapter.get_releases().set_row_data(index, card);
            });
            card
        })
        .collect()
}

fn release_card(release: &Release, artist: &Artist) -> CardAdapter {
    let release = release.clone();
    CardAdapter {
        image: ImageLinkAdapter {
            image: Default::default(),
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        title: LinkAdapter {
            name: release.title.clone().unwrap_or_default().into(),
            url: format!("dimple://release/{}", release.key.clone().unwrap_or_default()).into(),
            ..Default::default()
        },
        sub_title: LinkAdapter {
            name: artist.name.clone().unwrap_or_default().into(),
            url: format!("dimple://artist/{}", artist.key.clone().unwrap_or_default()).into(),
        },
        ..Default::default()
    }
}
