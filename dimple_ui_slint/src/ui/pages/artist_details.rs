use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Model;
use dimple_core::model::ReleaseGroup;
use slint::ComponentHandle;
use slint::Model as _;
use slint::ModelNotify;
use slint::ModelRc;
use slint::ModelTracker;
use url::Url;
use crate::ui::app_window_controller::App;
use crate::ui::Navigator;
use crate::ui::Page;
use crate::ui::ArtistDetailsAdapter;
use dimple_core::db::Db;
use crate::ui::CardAdapter;

pub fn artist_details(url: &str, app: &App) {
    let url = url.to_owned();
    let librarian = app.librarian.clone();
    let ui = app.ui.clone();
    let images = app.images.clone();
    std::thread::spawn(move || {        
        let url = Url::parse(&url).unwrap();
        let key = url.path_segments().unwrap().nth(0).unwrap();

        let artist: Artist = librarian.get(&Artist {
            key: Some(key.to_string()),
            ..Default::default()
        }.into()).unwrap().unwrap().into();

        let mut release_groups: Vec<ReleaseGroup> = librarian
            .list(&ReleaseGroup::default().into(), Some(&Model::Artist(artist.clone())))
            .unwrap()
            .map(Into::into)
            .collect();
        release_groups.sort_by_key(|f| f.first_release_date.to_owned());
        release_groups.reverse();

        ui.upgrade_in_event_loop(move |ui| {
            let albums: Vec<CardAdapter> = release_groups.iter().cloned()
                .filter(|release_group| release_group.primary_type == Some("album".to_string()))
                .enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_artist_details().albums.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_artist_details().albums.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let eps: Vec<CardAdapter> = release_groups.iter().cloned()
                .filter(|release_group| release_group.primary_type == Some("ep".to_string()))
                .enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_artist_details().eps.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_artist_details().eps.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let singles: Vec<CardAdapter> = release_groups.iter().cloned()
                .filter(|release_group| release_group.primary_type == Some("single".to_string()))
                .enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_artist_details().singles.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_artist_details().singles.set_row_data(index, card);
                    });
                    card
                })
                .collect();
            let others: Vec<CardAdapter> = release_groups.iter().cloned()
                // TODO not other, but inverse
                .filter(|release_group| release_group.primary_type == Some("other".to_string()))
                .enumerate()
                .map(|(index, release_group)| {
                    let mut card: CardAdapter = release_group.clone().into();
                    card.image.image = images.lazy_get(release_group.model(), 200, 200, move |ui, image| {
                        let mut card = ui.get_artist_details().others.row_data(index).unwrap();
                        card.image.image = image;
                        ui.get_artist_details().others.set_row_data(index, card);
                    });
                    card
                })
                .collect();

            let mut adapter = ArtistDetailsAdapter {
                card: artist.clone().into(),
                disambiguation: artist.disambiguation.clone().unwrap_or_default().into(),
                summary: artist.summary.clone().unwrap_or_default().into(),
                albums: ModelRc::from(albums.as_slice()),
                singles: ModelRc::from(singles.as_slice()),
                eps: ModelRc::from(eps.as_slice()),
                others: ModelRc::from(others.as_slice()),
                // genres: link_adapters(genres),
                // links: link_adapters(artist_links(&artist)),
                dump: serde_json::to_string_pretty(&artist).unwrap().into(),
                ..Default::default()
            };
            adapter.card.image.image = images.get(artist.model(), 275, 275);
            ui.set_artist_details(adapter);
            ui.set_page(Page::ArtistDetails);
            ui.global::<Navigator>().set_busy(false);
        }).unwrap();
    });
}

pub struct VecModel<T> {
    // the backing data, stored in a `RefCell` as this model can be modified
    array: std::cell::RefCell<Vec<T>>,
    // the ModelNotify will allow to notify the UI that the model changes
    notify: ModelNotify,
}

impl<T: Clone + 'static> slint::Model for VecModel<T> {
    type Data = T;

    fn row_count(&self) -> usize {
        self.array.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        self.array.borrow().get(row).cloned()
    }

    fn set_row_data(&self, row: usize, data: Self::Data) {
        self.array.borrow_mut()[row] = data;
        // don't forget to call row_changed
        self.notify.row_changed(row);
    }

    fn model_tracker(&self) -> &dyn ModelTracker {
        &self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        // a typical implementation just return `self`
        self
    }
}

// when modifying the model, we call the corresponding function in
// the ModelNotify
impl<T> VecModel<T> {
    /// Add a row at the end of the model
    pub fn push(&self, value: T) {
        self.array.borrow_mut().push(value);
        self.notify.row_added(self.array.borrow().len() - 1, 1)
    }

    /// Remove the row at the given index from the model
    pub fn remove(&self, index: usize) {
        self.array.borrow_mut().remove(index);
        self.notify.row_removed(index, 1)
    }
}

