use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Genre;
use dimple_core::model::Medium;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use dimple_librarian::librarian::Librarian;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelRc;
use slint::SharedString;
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

// TODO this is what is causing that little hiccup. Half a meg of JSON that
// I turn into thousands of objects and merge on every load. 
// Gotta stop doing that :)
// [2024-06-22T00:31:37.511Z INFO  dimple_librarian::plugin] MusicBrainz 496068 (Cached) https://musicbrainz.org/ws/2/release?fmt=json&offset=0&limit=100&release-group=0da580f2-6768-498f-af9d-2becaddf15e0&inc=artist-credits labels recordings release-groups media discids isrcs
// This is a good one too
// [2024-06-22T01:24:25.672Z INFO  dimple_librarian::plugin] MusicBrainz 543565 (Cached) https://musicbrainz.org/ws/2/release?fmt=json&offset=0&limit=100&release-group=9f7a4c28-8fa2-3113-929c-c47a9f7982c3&inc=artist-credits labels recordings release-groups media discids isrcs
pub fn release_group_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap().to_string();

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
        releases.sort_by_key(|release| release.date.clone().unwrap_or_default());

        let release = releases.get(0).unwrap().clone();
        let mut media_and_tracks: Vec<(Medium, Vec<Track>)> = vec![];
        if let Ok(Some(release)) = librarian.get(&release.model()) {
            let release: Release = release.into();
            media_and_tracks = librarian
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
        }

        ui.upgrade_in_event_loop(move |ui| {
            // let releases: Vec<CardAdapter> = releases.iter().cloned()
            //     .enumerate()
            //     .map(|(index, release)| {
            //         let mut card: CardAdapter = release.clone().into();
            //         card.image.image = images.lazy_get(release.model(), 200, 200, move |ui, image| {
            //             let mut card = ui.get_release_group_details().releases.row_data(index).unwrap();
            //             card.image.image = image;
            //             ui.get_release_group_details().releases.set_row_data(index, card);
            //         });
            //         card
            //     })
            //     .collect();

            let release_options: Vec<SharedString> = releases.iter().cloned()
                .map(|release| release_option(&release).into()).collect();

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
                key: key.clone().into(),
                card: release_group.clone().into(),
                artists: ModelRc::from(artists.as_slice()),
                disambiguation: release_group.disambiguation.clone().unwrap_or_default().into(),
                summary: release_group.summary.clone().unwrap_or_default().into(),
                // releases: ModelRc::from(releases.as_slice()),
                release_options: ModelRc::from(release_options.as_slice()),
                selected_release_option: release_options.get(0).unwrap().clone(),
                genres: ModelRc::from(genres.as_slice()),
                links: ModelRc::from(links.as_slice()),
                primary_type: release_group.primary_type.clone().unwrap_or_default().into(),
                media: ModelRc::from(media.as_slice()),
                dump: format!("{}\n{}",
                    serde_json::to_string_pretty(&release_group).unwrap(),
                    serde_json::to_string_pretty(&release).unwrap(),
                    ).into(),
                ..Default::default()
            };
            adapter.card.image.image = images.lazy_get(release_group.model(), 275, 275, |ui, image| {
                let mut model = ui.get_release_group_details();
                model.card.image.image = image;
                ui.set_release_group_details(model);
            });
            ui.set_release_group_details(adapter);
            ui.set_page(Page::ReleaseGroupDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

fn release_option(release: &Release) -> String {
    let release = release.clone();
    format!("{} {} {} {} {} {} ({})", 
        release.title.unwrap_or_default(),
        release.country.unwrap_or_default(),
        release.date.unwrap_or_default(),
        release.status.unwrap_or_default(),
        release.quality.unwrap_or_default(),
        release.primary_type.unwrap_or_default(),
        release.disambiguation.unwrap_or_default())
}

pub fn release_group_details_release_selected(app: &App, s: String) {
    let app = app.clone();
    let librarian = app.librarian.clone();
    app.ui.upgrade_in_event_loop(move |ui| {
        let key = ui.get_release_group_details().key.to_string();
        let release_group: ReleaseGroup = librarian.get(&ReleaseGroup {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();
        let mut releases: Vec<Release> = librarian
            .list(&Release::default().into(), &Some(release_group.model()))
            .unwrap()
            .map(Into::into)
            .collect();
        releases.sort_by_key(|release| release.date.clone().unwrap_or_default());

        let release = releases.iter()
            .find(|release| release_option(release) == s)
            .unwrap_or(releases.get(0).unwrap());
        let mut media_and_tracks: Vec<(Medium, Vec<Track>)> = vec![];
        if let Ok(Some(release)) = librarian.get(&release.model()) {
            let release: Release = release.into();
            media_and_tracks = librarian
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
        }

        let media: Vec<MediumAdapter> = media_and_tracks.iter().map(|(medium, tracks)| {
            MediumAdapter {
                title: medium.title.clone().unwrap_or_default().into(),
                tracks: track_adapters(tracks.to_vec()),
            }
        }).collect();

        let mut adapter = ui.get_release_group_details();
        adapter.selected_release_option = s.into();
        adapter.media = ModelRc::from(media.as_slice());
        adapter.dump = format!("{}\n{}",
            serde_json::to_string_pretty(&release_group).unwrap(),
            serde_json::to_string_pretty(&release).unwrap(),
            ).into();
        ui.set_release_group_details(adapter);
    }).unwrap();
}

// Note, looking at https://musicbrainz.org/ws/2/release-group/f44f4f73-a714-31a1-a4b8-bfcaaf311f50?inc=aliases%2Bartist-credits%2Breleases&fmt=json
// I noticed that release_group.first_release_date and release.date are the same for the "correct" default
// release. They match exactly where other dates don't. So that may be a good indicator too.
fn get_preferred_release(librarian: &Librarian, release_group: &ReleaseGroup) -> Option<Release> {
    // TODO
    // status = official
    // quality = high
    // date = oldest
    // track count >= 0

    let mut releases: Vec<Release> = librarian
        .list(&Release::default().into(), &Some(release_group.model()))
        .unwrap()
        .map(Into::into)
        .collect();
    releases.sort_by_key(|release| release.date.clone().unwrap_or_default());

    // for release in releases.iter() {
    //     println!("{} {:?} {:?} {:?} {:?}", 
    //         score_release(release), 
    //         release.country, 
    //         release.date, 
    //         release.status, 
    //         release.title);
    // }

    releases.get(0).cloned()
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

#[cfg(test)]
mod tests {
    #[test]
    fn preferred_release() {

    }
}