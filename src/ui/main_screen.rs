use std::{sync::{Arc}, collections::VecDeque};

use eframe::{egui::{self, Context, LayerId, Frame, Margin, ScrollArea}, epaint::{Mesh, Shape}};


use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library}};

use super::{nav_bar::{NavBar, NavEvent}, player_bar::PlayerBar, card_grid::{CardGrid, Card, LibraryItem}, item_details::ItemDetails, theme::Theme};

pub struct MainScreen {
    librarian: Arc<Librarian>,
    _player: PlayerHandle,

    nav_bar: NavBar,
    card_grid: CardGrid,
    item_details: ItemDetails,
    player_bar: PlayerBar,

    history: VecDeque<HistoryItem>,
}

pub enum HistoryItem {
    ItemDetails(LibraryItem),
    Search(String),
    Home,
}

// TODO I've gone this direction of having deep components bubble up their
// events but man it's a pain in the ass. Need to rethink the top level of the
// app and decide if the rest of the UI can just call, like, "navigate"

// Okay, so, yes Dimple::navigate(), that builds into the URL idea (ala
// dimple://artist/album). 
// But also I think it's time to split at least the content area into three
// components: dashboard, search results, item details. 
impl MainScreen {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        Self {
            librarian: librarian.clone(),
            _player: player.clone(),
            nav_bar: NavBar::default(),
            card_grid: CardGrid::default(),
            player_bar: PlayerBar::new(player.clone()),
            // cards: Vec::new(),
            history: VecDeque::new(),
            item_details: ItemDetails::new(player, librarian),
        }
    }

    pub fn ui(&mut self, ctx: &Context) {     
        let theme = Theme::get(ctx);

        self.gradient_background(ctx);

        // egui::Window::new("Settings").show(ctx, |ui| {
        //     ctx.settings_ui(ui);
        // });

        // egui::Window::new("Inspection").show(ctx, |ui| {
        //     ctx.inspection_ui(ui);
        // });

        // egui::Window::new("Memory").show(ctx, |ui| {
        //     ctx.memory_ui(ui);
        // });

        // egui::Window::new("Textures").show(ctx, |ui| {
        //     ctx.texture_ui(ui);
        // });

        // ctx.set_debug_on_hover(true);

        egui::TopBottomPanel::top("nav_bar")
            .frame(Frame::none().inner_margin(Margin { top: 16.0, right: 8.0, bottom: 8.0, left: 8.0, }))
            .show_separator_line(false)
            .show(ctx, |ui| {
                match self.nav_bar.ui(ui) {
                    Some(NavEvent::Back) => {
                        self.history.pop_front();
                    },
                    Some(NavEvent::Home) => {
                        self.history.push_front(HistoryItem::Home);
                    },
                    Some(NavEvent::Search(query)) => {
                        self.history.push_front(HistoryItem::Search(query));
                    },
                    None => (),
                }
            });
        
        egui::TopBottomPanel::bottom("player")
            .frame(Frame::none().inner_margin(Margin::same(8.0)).fill(theme.player_background))
            .show(ctx, |ui| {
                if let Some(item) = self.player_bar.ui(ui) {
                    self.history.push_front(HistoryItem::ItemDetails(item));
                }
            });
        
        egui::CentralPanel::default()
            .frame(Frame::none().inner_margin(Margin::same(8.0)))
            .show(ctx, |ui| {
                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    match self.history.front() {
                        Some(HistoryItem::Home) | None => {
                            // TODO Clear search bar query string
                            let cards = self.home();
                            let action = self.card_grid.ui("home", &cards, 200.0, 200.0, ui);
                            if let Some(library_item) = action {
                                self.history.push_front(HistoryItem::ItemDetails(library_item));
                            }
                        },
                        Some(HistoryItem::Search(query)) => {
                            // TODO can't run the query every frame
                            // TODO we should set the search bar query string when
                            // showing this.
                            for (category, cards) in self.search(query.clone().as_str()) {
                                if cards.is_empty() {
                                    continue;
                                }
                                ui.label(Theme::heading(&category));
                                let action = self.card_grid.ui(&category, &cards, 200.0, 200.0, ui);
                                if let Some(library_item) = action {
                                    self.history.push_front(HistoryItem::ItemDetails(library_item));
                                }
                                ui.add_space(32.0);
                            }
                        },
                        Some(HistoryItem::ItemDetails(item)) => {
                            if let Some(library_item) = self.item_details.ui(item.clone(), ui) {
                                self.history.push_front(HistoryItem::ItemDetails(library_item));
                            }
                        },
                    }
                });
        });
    }

    pub fn home(&self) -> Vec<Box<dyn Card>>  {
        self.librarian.genres()
            .into_iter()
            .map(Into::into)
            .collect::<Vec<Box<dyn Card>>>()
    }
    
    fn search(&self, query: &str) -> Vec<(String, Vec<Box<dyn Card>>)> {
        let matcher = SkimMatcherV2::default();
        let artists = self.librarian.artists().into_iter()
            .filter(|artist| matcher.fuzzy_match(&artist.name, query).is_some())
            .map(Into::into)
            .collect::<Vec<Box<dyn Card>>>();
        let mut releases = self.librarian.releases().into_iter()
            .filter(|release| matcher.fuzzy_match(&release.title, query).is_some())
            .map(Into::into)
            .collect::<Vec<Box<dyn Card>>>();
        releases.sort_by_key(|release| release.title());
        let genres = self.librarian.genres().into_iter()
            .filter(|genre| matcher.fuzzy_match(&genre.name, query).is_some())
            .map(Into::into)
            .collect::<Vec<Box<dyn Card>>>();
        vec![
            ("Artists".to_owned(), artists), 
            ("Releases".to_owned(), releases), 
            ("Genres".to_owned(), genres)
        ]
    }

    fn gradient_background(&mut self, ctx: &Context) {
        let theme = Theme::get(ctx);

        let top = theme.background_top;
        let middle = theme.background_middle;
        let bottom = theme.background_bottom;

        let painter = ctx.layer_painter(LayerId::background());
        let mut mesh = Mesh::default();
        let rect = painter.clip_rect();
        
        mesh.colored_vertex(rect.left_top(), top);
        mesh.colored_vertex(rect.right_top(), top);
        mesh.colored_vertex(rect.right_center(), middle);
        mesh.colored_vertex(rect.left_center(), middle);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);

        mesh.colored_vertex(rect.left_center(), middle);
        mesh.colored_vertex(rect.right_center(), middle);
        mesh.colored_vertex(rect.right_bottom(), bottom);
        mesh.colored_vertex(rect.left_bottom(), bottom);
        mesh.add_triangle(4, 5, 6);
        mesh.add_triangle(4, 6, 7);

        painter.add(Shape::Mesh(mesh));
    }
}

