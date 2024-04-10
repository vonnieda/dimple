use std::rc::Rc;

use crate::ui::AppWindow;
use crate::ui::TrackListAdapter;
use crate::ui::Page;
use dimple_librarian::librarian::Librarian;
use slint::ModelRc;
use slint::StandardListViewItem;
use slint::VecModel;

pub fn track_list(_librarian: &Librarian, ui: slint::Weak<AppWindow>) {
    ui.upgrade_in_event_loop(move |ui| {
        let rows: VecModel<ModelRc<StandardListViewItem>> = VecModel::default();
        for i in 0..100 {
            let row = Rc::new(VecModel::default());
            row.push(StandardListViewItem::from(format!("{}", i).as_str()));
            row.push(StandardListViewItem::from(format!("{}", i).as_str()));
            row.push(StandardListViewItem::from(format!("{}", i).as_str()));
            row.push(StandardListViewItem::from(format!("{}", i).as_str()));
            row.push(StandardListViewItem::from(format!("{}", i).as_str()));
            row.push(StandardListViewItem::from(format!("{}", i).as_str()));
            rows.push(row.into());
        }
        let adapter = TrackListAdapter {
            rows: ModelRc::new(rows),
        };
        ui.set_track_list(adapter);
        ui.set_page(Page::TrackList);
    })
    .unwrap();
}

