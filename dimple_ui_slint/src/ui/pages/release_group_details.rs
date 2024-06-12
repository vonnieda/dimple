use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Medium;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use dimple_core::db::Db;
use crate::ui::CardAdapter;
use crate::ui::LinkAdapter;
use crate::ui::ReleaseGroupDetailsAdapter;
use crate::ui::TrackAdapter;
use crate::ui::MediumAdapter;

pub fn release_group_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let release_group: ReleaseGroup = librarian.get(&ReleaseGroup {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        let mut artists: Vec<Artist> = librarian
            .list(&Artist::default().into(), &Some(release_group.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        artists.sort_by_key(|f| f.name.to_owned());

        let mut genres: Vec<Genre> = librarian
            .list(&Genre::default().into(), &Some(release_group.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        genres.sort_by_key(|f| f.name.to_owned());

        let mut releases: Vec<Release> = librarian
            .list(&Release::default().into(), &Some(release_group.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        releases.sort_by(|l, r| score_release(l).total_cmp(&score_release(r)));
        releases.reverse();

        // for release in releases.iter() {
        //     println!("{} {:?} {:?} {:?} {:?}", 
        //         score_release(release), 
        //         release.country, 
        //         release.date, 
        //         release.status, 
        //         release.title);
        // }
        
        let release = releases.get(0).unwrap();
        // TODO not sure how I feel about this yet. Makes it slow.
        let release: Release = librarian.get(&release.model()).unwrap().unwrap().into();
        let media_and_tracks: Vec<_> = librarian
            .list(&Medium::default().into(), &Some(release.model()))
            .unwrap()
            .map(Into::<Medium>::into)
            .map(|medium| {
                let tracks: Vec<Track> = librarian.list(&Track::default().model(), &Some(medium.model()))
                    .unwrap()
                    .map(Into::<Track>::into)
                    .collect();
                (medium, tracks)
            })
            .collect();

        ui.upgrade_in_event_loop(move |ui| {
            let releases: Vec<CardAdapter> = releases.iter().cloned()
                .enumerate()
                .map(|(index, release)| {
                    let mut card: CardAdapter = release.clone().into();
                    card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_release_group_details().releases.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_release_group_details().releases.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            let artists: Vec<LinkAdapter> = artists.iter().cloned().map(|artist| {
                LinkAdapter {
                    name: artist.name.unwrap().into(),
                    url: format!("dimple://artist/{}", artist.key.unwrap()).into(),
                }
            }).collect();

            let genres: Vec<LinkAdapter> = genres.iter().cloned().map(|genre| {
                LinkAdapter {
                    name: genre.name.unwrap().into(),
                    url: format!("dimple://genre/{}", genre.key.unwrap()).into(),
                }
            }).collect();

            let links: Vec<LinkAdapter> = release_group.links.iter().map(|link| {
                LinkAdapter {
                    name: link.into(),
                    url: link.into(),
                }
            }).collect();

            let media: Vec<MediumAdapter> = media_and_tracks.iter().map(|(medium, tracks)| {
                MediumAdapter {
                    title: medium.title.clone().unwrap_or_default().into(),
                    tracks: track_adapters(tracks.to_vec()),
                }
            }).collect();

            let mut adapter = ReleaseGroupDetailsAdapter {
                card: release_group.clone().into(),
                artists: ModelRc::from(artists.as_slice()),
                disambiguation: release_group.disambiguation.clone().unwrap_or_default().into(),
                summary: release_group.summary.clone().unwrap_or_default().into(),
                releases: ModelRc::from(releases.as_slice()),
                genres: ModelRc::from(genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                primary_type: release_group.primary_type.clone().unwrap_or_default().into(),
                media: ModelRc::from(media.as_slice()),
                dump: serde_json::to_string_pretty(&release_group).unwrap().into(),                
                ..Default::default()
            };
            adapter.card.image.image = images.get(release_group.model(), 275, 275);
            ui.set_release_group_details(adapter);
            ui.set_page(Page::ReleaseGroupDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

fn track_adapters(tracks: Vec<Track>) -> ModelRc<TrackAdapter> {
    let adapters: Vec<_> = tracks.iter()
        .map(|t| TrackAdapter {
            title: LinkAdapter {
                name: t.title.clone().unwrap_or_default().into(),
                url: format!("dimple://track/{}", t.key.clone().unwrap_or_default()).into(),
                ..Default::default()
            },
            track_number: format!("{}", t.number.clone().unwrap_or_default()).into(),
            length: length_to_string(t.length.clone().unwrap_or_default()).into(),
            // artists: Default::default(),
            // plays: 0,
            // source_count: t.sources.len() as i32,
            ..Default::default()
        })
        .collect();
    ModelRc::from(adapters.as_slice())
}

fn length_to_string(length: u32) -> String {
    format!("{}:{:02}", 
        length / (60 * 1000), 
        length % (60 * 1000) / 1000)
}

fn get_preferred_release(releases: &[Release]) -> Option<Release> {
    // worldwide digital release is best
    // anything digital is next
    // anything wordlwide is next
    todo!()
}

// Creates a simple score for a release to use when selecting a
// a default release.
// TODO this is super naive, just needed something to set the example.
fn score_release(r: &Release) -> f64 {
    let mut score = 0.;
    let country = r.country.clone().unwrap_or_default().to_lowercase();
    if country == "xw" {
        score += 1.0;
    }                
    else if country == "us" || country == "gb" || country == "xe" {
        score += 0.7;
    }
    else if !country.is_empty() {
        score += 0.1;
    }

    if r.status.clone().unwrap_or_default().to_lowercase() == "official" {
        score += 1.0;
    }

    let packaging = r.packaging.clone().unwrap_or_default().to_lowercase();
    if packaging == "digipak" {
        score += 1.0;
    }
    else if packaging == "jewelcase" {
        score += 0.5;
    }

    if !r.media.is_empty() {
        let mut media_format_score = 0.;
        for media in r.media.clone() {
            let format = media.format.unwrap_or_default().to_lowercase();
            if format == "digital media" {
                media_format_score += 1.0;
            }
            else if format == "cd" {
                media_format_score += 0.5;
            }
        }
        score += media_format_score / r.media.len() as f64;
    }

    score / 4.
}
