use std::{sync::{Arc, RwLock}};

use eframe::{egui::{self, Context, ImageButton, Ui, Link, RichText, LayerId, Id, TextStyle}, epaint::{Color32, FontId, Mesh, Shape, Rect, Stroke}};

use egui_extras::RetainedImage;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::{player::PlayerHandle, librarian::Librarian, music_library::{Library, Release, Artist, Genre, Playlist}, dimple::Theme};

use super::{search_bar::SearchBar, player_bar::PlayerBar, card_grid::{CardGrid, Card, LibraryItem}, retained_images::RetainedImages, item_details::ItemDetails};

pub struct MainScreen {
    librarian: Arc<Librarian>,
    player: PlayerHandle,
    retained_images: Arc<RetainedImages>,

    search_bar: SearchBar,
    card_grid: CardGrid,
    player_bar: PlayerBar,
    last_rect: Option<Rect>,
    cards: Vec<Box<dyn Card>>,

    selected_item: Option<LibraryItem>,
    item_details: ItemDetails,
}

// Artist Cards
// Genre Cards
// Playlist Cards
// TODO Release Details
// TODO Artist Details
// TODO Genre Details

impl MainScreen {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        let retained_images = Arc::new(RetainedImages::new(librarian.clone()));
        let mut main_screen = Self {
            librarian: librarian.clone(),
            player: player.clone(),
            retained_images: retained_images.clone(),
            search_bar: SearchBar::default(),
            card_grid: CardGrid::default(),
            player_bar: PlayerBar::new(player.clone(), retained_images.clone()),
            cards: Vec::new(),
            selected_item: None,
            item_details: ItemDetails::new(retained_images.clone()),
            last_rect: None,
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

        egui::Window::new("Style").show(ctx, |ui| {
            ctx.style_ui(ui);
        });

        // ctx.set_debug_on_hover(true);

        egui::TopBottomPanel::top("search_bar").show(ctx, |ui| {
            ui.add_space(16.0);
            if self.search_bar.ui(ctx, ui).changed() {
                let query = self.search_bar.query.clone();
                self.cards = self.cards(&query);
                self.selected_item = None;
            }
            ui.add_space(8.0);
        });
        
        let panel = egui::TopBottomPanel::bottom("player").show(ctx, |ui| {
            if let Some(last_rect) = self.last_rect {
                let painter = ui.painter();
                painter.rect_filled(last_rect, 0.0, Theme::player_background);
                painter.line_segment([last_rect.left_top(), last_rect.right_top()], Stroke::new(1.0, Color32::from_gray(0xc3)));
            }
            ui.vertical(|ui| {
                self.player_bar.ui(ctx, ui);
                ui.add_space(8.0);
            });
        });
        self.last_rect = Some(panel.response.rect);
        
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(item) = &self.selected_item {
                self.item_details.ui(item.clone(), ctx, ui);
            }
            else {
                let action = self.card_grid.ui(&self.cards, 200.0, 200.0, ctx, ui);
                if action.is_some() {
                    self.selected_item = action;
                }
            }
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
            ui.spacing_mut().button_padding[0] = 2.0;
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

