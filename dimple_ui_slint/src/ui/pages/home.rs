use crate::ui::app_window_controller::App;
use crate::ui::Page;

pub fn home_init(app: &App) {
}

pub fn home(app: &App) {
    let app = app.clone();
    std::thread::spawn(move || {
        app.ui.upgrade_in_event_loop(move |ui| {
            ui.set_page(Page::Home);
        })
        .unwrap();
    });
}



// pub struct HomePage {
// }

// impl HomePage {

// }

// /// Via listening history:
// /// - Seasons
// /// - Holidays
// /// - Birthday
// /// - Obessions
// /// - Old favorites
// impl Page for HomePage {
//     fn navigate(librarian: &Librarian, ui: slint::Weak<AppWindow>) {
//         let librarian = librarian.clone();
//         std::thread::spawn(move || {
//             let mut tracks: Vec<(Artist, Release, Recording, RecordingSource)> = Vec::new();
//             let artists = librarian
//                 .list(&Artist::default().into(), None, &AccessMode::Online).unwrap()
//                 .map(Into::<Artist>::into);
//             for artist in artists {
//                 dbg!(artist);
//                 // for release in artist.releases(librarian.as_ref()) {
//                 //     for recording in release.recordings(librarian.as_ref()) {
//                 //         for source in recording.sources(librarian.as_ref()) {
//                 //             tracks.push((artist.clone(), release.clone(), recording.clone(), source.clone()));
//                 //         }
//                 //     }
//                 // }
//             }
    
//             ui.upgrade_in_event_loop(move |ui| {
//                 let rows: VecModel<ModelRc<StandardListViewItem>> = VecModel::default();
//                 for (artist, release, recording, source) in tracks {
//                     let row = Rc::new(VecModel::default());
//                     row.push(recording.title.listview_item());
//                     row.push(release.title.listview_item());
//                     row.push(artist.name.listview_item());
//                     row.push(source.extension.listview_item());
//                     row.push(format!("{:?}", source.source_ids).listview_item());
//                     row.push("".listview_item());
//                     rows.push(row.into());
//                 }
//                 let adapter = HomeAdapter {
//                     rows: ModelRc::new(rows),
//                 };
//                 ui.set_home_adapter(adapter);
//                 ui.set_page(5);
//             }).unwrap();
//         });
//     }    
// }

