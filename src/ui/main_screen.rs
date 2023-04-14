use std::{sync::{Arc, RwLock}, collections::VecDeque};

use eframe::{egui::{self, Context, ImageButton, Ui, LayerId, Frame, Margin}, epaint::{Color32, Mesh, Shape, Rect, Stroke}};

use egui_extras::RetainedImage;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library, Release}, dimple::Theme};

use super::{nav_bar::{NavBar, NavEvent}, player_bar::PlayerBar, card_grid::{CardGrid, Card, LibraryItem}, retained_images::RetainedImages, item_details::ItemDetails};

pub struct MainScreen {
    librarian: Arc<Librarian>,
    player: PlayerHandle,
    retained_images: Arc<RetainedImages>,

    nav_bar: NavBar,
    card_grid: CardGrid,
    item_details: ItemDetails,
    player_bar: PlayerBar,

    player_last_rect: Option<Rect>,
    cards: Vec<Box<dyn Card>>,

    history: VecDeque<HistoryItem>,
}

pub enum HistoryItem {
    ItemDetails(LibraryItem),
    Search(String),
    Home,
}

// TODO Artist Cards
// TODO Genre Cards
// TODO Playlist Cards
impl MainScreen {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        let retained_images = Arc::new(RetainedImages::new(librarian.clone()));
        let mut main_screen = Self {
            librarian: librarian.clone(),
            player: player.clone(),
            retained_images: retained_images.clone(),
            nav_bar: NavBar::default(),
            card_grid: CardGrid::default(),
            player_bar: PlayerBar::new(player.clone(), retained_images.clone()),
            cards: Vec::new(),
            history: VecDeque::new(),
            item_details: ItemDetails::new(retained_images.clone(), player.clone()),
            player_last_rect: None,
        };
        main_screen.cards = main_screen.cards("");
        main_screen
    }

    fn gradient_background(&mut self, ctx: &Context) {
        // let painter = ctx.layer_painter(LayerId::new(egui::Order::PanelResizeLine, Id::new("gradient")));
        let painter = ctx.layer_painter(LayerId::background());
        let mut mesh = Mesh::default();
        let rect = painter.clip_rect();
        let top = Theme::background_top;
        let middle = Theme::background_middle;
        let bottom = Theme::background_bottom;
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

    pub fn ui(&mut self, ctx: &Context) {        
        self.gradient_background(ctx);

        // egui::Window::new("Style").show(ctx, |ui| {
        //     ctx.style_ui(ui);
        // });

        // ctx.set_debug_on_hover(true);

        egui::TopBottomPanel::top("nav_bar").show(ctx, |ui| {
            Frame::none()
            .inner_margin(Margin {
                left: 8.0,
                right: 0.0,
                top: 16.0,
                bottom: 0.0,
            })
            .show(ui, |ui| {
                match self.nav_bar.ui(ctx, ui) {
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
        });
        
        let panel = egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            if let Some(last_rect) = self.player_last_rect {
                let painter = ui.painter();
                painter.rect_filled(last_rect, 0.0, Theme::player_background);
                painter.line_segment([last_rect.left_top(), last_rect.right_top()], Stroke::new(1.0, Color32::from_gray(0xc3)));
            }
            Frame::none().inner_margin(Margin {
                left: 8.0,
                right: 8.0,
                top: 2.0,
                bottom: 10.0,
            }).show(ui, |ui| {
                if let Some(item) = self.player_bar.ui(ctx, ui) {
                    self.history.push_front(HistoryItem::ItemDetails(item));
                }
            });
        });
        self.player_last_rect = Some(panel.response.rect);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            Frame::none()
                .inner_margin(Margin {
                    left: 8.0,
                    right: 8.0,
                    top: 8.0,
                    bottom: 8.0,
                })
                .show(ui, |ui| {
                    match self.history.front() {
                        Some(HistoryItem::ItemDetails(item)) => {
                            if let Some(library_item) = self.item_details.ui(item.clone(), ctx, ui) {
                                self.history.push_front(HistoryItem::ItemDetails(library_item));
                            }
                        },
                        // TODO screens showing lists should auto update
                        Some(HistoryItem::Search(query)) => {
                            // TODO can't run the query every frame
                            self.cards = self.cards(query.clone().as_str());
                            let action = self.card_grid.ui(&self.cards, 200.0, 200.0, ctx, ui);
                            if let Some(library_item) = action {
                                self.history.push_front(HistoryItem::ItemDetails(library_item));
                            }
                        },
                        Some(HistoryItem::Home) => {
                            // TODO can't run the query every frame
                            self.cards = self.cards("");
                            let action = self.card_grid.ui(&self.cards, 200.0, 200.0, ctx, ui);
                            if let Some(library_item) = action {
                                self.history.push_front(HistoryItem::ItemDetails(library_item));
                            }
                        },
                        None => {
                            // TODO can't run the query every frame
                            self.cards = self.cards("");
                            let action = self.card_grid.ui(&self.cards, 200.0, 200.0, ctx, ui);
                            if let Some(library_item) = action {
                                self.history.push_front(HistoryItem::ItemDetails(library_item));
                            }
                        },
                    }
                });
        });
    }

    /// Get the list of Cards to show in the grid. Performs filtering, sorting,
    /// and caching.
    fn cards(&mut self, query: &str) -> Vec<Box<dyn Card>> {
        // Filter Releases by the query
        let matcher = SkimMatcherV2::default();
        let mut releases: Vec<Release> = self.librarian.releases().into_iter()
            .filter(|release| {
                let haystack = format!("{} {}", release.title, release.artist())
                    .to_uppercase();
                return matcher
                    .fuzzy_match(haystack.as_str(), query)
                    .is_some();
            })
            .collect();

        // Sort Releases by Artist Name then Release Title
        releases.sort_by(|a, b| {
            a.artist().to_uppercase()
                .cmp(&b.artist().to_uppercase())
                .then(a.title.to_uppercase().cmp(&b.title.to_uppercase()))
        });

        // Convert to Cards
        releases.into_iter()
            .map(|release| {
                Box::new(self.card_from_release(&release)) as Box<dyn Card>
            })
            .collect()
    }

    fn card_from_release(&mut self, release: &Release) -> ReleaseCard {
        ReleaseCard {
            release: release.clone(),
            image: self.retained_images.get(release.art.first().unwrap(), 200, 200),
            player: self.player.clone(),
        }
    }
}

pub struct ReleaseCard {
    release: Release,
    image: Arc<RwLock<Arc<RetainedImage>>>,
    player: PlayerHandle,
}

impl ReleaseCard {
}

impl Card for ReleaseCard {
    fn ui(&self, image_width: f32, image_height: f32, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            let image_button =
                ImageButton::new(self.image.read().unwrap().texture_id(ctx), 
                    egui::vec2(image_width, image_height));
            // art
            if ui.add(image_button).clicked() {
                action = Some(LibraryItem::Release(self.release.clone()));
            }
            // title
            if ui.link(Theme::big_n_bold(&self.release.title)).clicked() {
                action = Some(LibraryItem::Release(self.release.clone()));
            }
            // Show each artist as a clickable link separated by commas
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = [0.0, 0.0].into();
                let len = self.release.artists.len();
                for (i, artist) in self.release.artists.iter().enumerate() {
                    if ui.link(&self.release.artist()).clicked() {
                        action = Some(LibraryItem::Artist(artist.clone()));
                    }
                    if i < len - 1 {
                        ui.label(",");
                    }
                }
            });
        });
        action
    }   
}

