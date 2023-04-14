use std::{sync::Arc};

use eframe::{egui::{Context, Ui, Color32, Response, ImageButton, Frame, Margin}, epaint::Stroke};
use egui_extras::RetainedImage;

use crate::{music_library::{Image, Artist, Genre, Release, Track, Playlist, Library}, dimple::Theme, player::PlayerHandle, librarian::Librarian};

use super::{card_grid::{LibraryItem, CardGrid, Card}, utils, retained_images::RetainedImages, main_screen::ReleaseCard};

// TODO feels like it's time to split this up
pub struct ItemDetails {
    retained_images: Arc<RetainedImages>,
    player: PlayerHandle,
    librarian: Arc<Librarian>,

    artist_icon: RetainedImage,
    release_icon: RetainedImage,
    track_icon: RetainedImage,
    genre_icon: RetainedImage,
    favorite_icon: RetainedImage,

    play_icon: RetainedImage,    
    add_icon: RetainedImage,
}

// TODO Clearly time to either pass "App" around, or AppContext with all this stuff. Everything needs it.
impl ItemDetails {
    pub fn new(retained_images: Arc<RetainedImages>, player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        Self {
            retained_images,
            player,
            librarian,

            artist_icon: Theme::svg_icon(include_bytes!("../icons/material/group_FILL0_wght400_GRAD0_opsz48.svg")),
            release_icon: Theme::svg_icon(include_bytes!("../icons/material/album_FILL0_wght400_GRAD0_opsz48.svg")),
            track_icon: Theme::svg_icon(include_bytes!("../icons/material/music_note_FILL0_wght400_GRAD0_opsz48.svg")),
            genre_icon: Theme::svg_icon(include_bytes!("../icons/material/theater_comedy_FILL0_wght400_GRAD0_opsz48.svg")),
            favorite_icon: Theme::svg_icon(include_bytes!("../icons/material/favorite_FILL0_wght400_GRAD0_opsz48.svg")),

            play_icon: Theme::svg_icon(include_bytes!("../icons/material/play_circle_FILL0_wght400_GRAD0_opsz48.svg")),
            add_icon: Theme::svg_icon(include_bytes!("../icons/material/add_circle_FILL0_wght400_GRAD0_opsz48.svg")),
        }
    }

    // Links for artist(s), release, genre(s)
    // So, for each kind of thing, what do we show?
    // 
    /// Release(release)
    ///     Vertical
    ///         Horizontal
    ///             Carousel(art, 300, 300)
    ///             Vertical
    ///                 Horizontal
    ///                     ImageButton(release_icon)
    ///                     Heading(release.title)
    ///                 Horizontal
    ///                     ImageButton(artist_icon)
    ///                     Links(release.artists)
    ///                 Horizontal
    ///                     ImageButton(genre_icon)
    ///                     Links(release.genres)
    ///         CardGrid(more_like_this)
    ///     
    // Artist: Art(Carousel), Name, Genre(s), Grid(Releases), Grid(More Artists Like This)
    // Track: Art(Carousel), Title, Lyrics
    pub fn ui(&mut self, item: LibraryItem, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        use LibraryItem::*;

        match item {
            Release(release) => self.release(&release, ctx, ui),
            Artist(artist) => self.artist(&artist, ctx, ui),
            Genre(genre) => self.genre(&genre, ctx, ui),
            Playlist(playlist) => self.playlist(&playlist, ctx, ui),
            Track(track) => self.track(&track, ctx, ui),
        }
    }

    pub fn release(&mut self, release: &Release, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.carousel(&release.art, 250, 250, ctx, ui);
                        self.play_controls(&LibraryItem::Release(release.clone()), ctx, ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&release.title);
                    });
                    ui.horizontal(|ui| {
                        ui.label("by");
                        if let Some(item) = self.artist_links(&release.artists, ctx, ui) {
                            action = Some(item);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("in");
                        if let Some(item) = self.genre_links(&release.genres, ctx, ui) {
                            action = Some(item);
                        }
                    });
                });
            });
        });
        action
    }

    pub fn artist(&mut self, artist: &Artist, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.carousel(&artist.art, 250, 250, ctx, ui);
                        self.play_controls(&LibraryItem::Artist(artist.clone()), ctx, ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&artist.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("in");
                        self.genre_links(&artist.genres, ctx, ui);
                    });
                })
            });
            let cards = self.release_cards_by_artist(artist);
            if let Some(item) = CardGrid::default().ui(&cards, 100.0, 100.0, ctx, ui) {
                action = Some(item);
            }
        });
        action
    }

    pub fn genre(&mut self, genre: &Genre, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.carousel(&genre.art, 250, 250, ctx, ui);
                        self.play_controls(&LibraryItem::Genre(genre.clone()), ctx, ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&genre.name);
                    });
                })
            });
            let cards = self.release_cards_by_genre(genre);
            if let Some(item) = CardGrid::default().ui(&cards, 100.0, 100.0, ctx, ui) {
                action = Some(item);
            }
        });
        action
    }


    // TODO okay scratched this together real quick to see it in action
    // and it's glorious. Little cards of releases under artist. But it exposes
    // the need for several things, like Librarian, to be globalish. 
    // And maybe helper functions for like Dimple::card_from<T>(T t) for Library
    // Items.
    // Mostly need a way to do queries on releases and turn them into cards.
    fn release_cards_by_artist(&mut self, artist: &Artist) -> Vec<Box<dyn Card>> {
        let mut releases: Vec<Release> = self.librarian.releases().into_iter()
            .filter(|release| {
                release.artists.contains(artist)
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

    fn release_cards_by_genre(&mut self, genre: &Genre) -> Vec<Box<dyn Card>> {
        let mut releases: Vec<Release> = self.librarian.releases().into_iter()
            .filter(|release| {
                release.genres.contains(genre)
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

    pub fn playlist(&mut self, playlist: &Playlist, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.carousel(&playlist.art, 250, 250, ctx, ui);
                        self.play_controls(&LibraryItem::Playlist(playlist.clone()), ctx, ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&playlist.name);
                    });
                })
            })
        });
        None
    }

    pub fn track(&mut self, track: &Track, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        self.carousel(&track.art, 250, 250, ctx, ui);
                        self.play_controls(&LibraryItem::Track(track.clone()), ctx, ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&track.title);
                    });
                    ui.horizontal(|ui| {
                        ui.label("by");
                        if let Some(item) = self.artist_links(&track.artists, ctx, ui) {
                            action = Some(item);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("in");
                        self.genre_links(&track.genres, ctx, ui);
                    });
                })
            })
        });
        action
    }

    pub fn play_controls(&self, library_item: &LibraryItem, ctx: &Context, ui: &mut Ui) {
        if Theme::icon_button(&self.add_icon, 48, 48, ctx, ui).clicked() {
            match library_item {
                LibraryItem::Release(release) => {
                    self.player.write().unwrap().queue_release(release);
                },
                LibraryItem::Artist(_) => todo!(),
                LibraryItem::Genre(_) => todo!(),
                LibraryItem::Playlist(_) => todo!(),
                LibraryItem::Track(track) => todo!(),
            }
        }
    }

    pub fn artist_links(&self, artists: &Vec<Artist>, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        // Show each artist as a clickable link separated by commas
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = [0.0, 0.0].into();
            let len = artists.len();
            for (i, artist) in artists.iter().enumerate() {
                if ui.link(&artist.name).clicked() {
                    action = Some(LibraryItem::Artist(artist.clone()));
                }
                if i < len - 1 {
                    ui.label(",");
                }
            }
        });
        action
    }

    pub fn genre_links(&self, genres: &Vec<Genre>, ctx: &Context, ui: &mut Ui) -> Option<LibraryItem> {
        // Show each genre as a clickable link separated by commas
        let mut action = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = [0.0, 0.0].into();
            let len = genres.len();
            for (i, genre) in genres.iter().enumerate() {
                if ui.link(&genre.name).clicked() {
                    action = Some(LibraryItem::Genre(genre.clone()));
                }
                if i < len - 1 {
                    ui.label(",");
                }
            }
        });
        action
    }

    // TODO actually carousel
    pub fn carousel(&mut self, images: &Vec<Image>, width: usize, height: usize, ctx: &Context, ui: &mut Ui) {
        let texture_id = match images.first() {
            Some(image) => self.retained_images.get(image, width as usize, height as usize)
                .read()
                .unwrap()
                .texture_id(ctx),
            None => utils::sample_image(Color32::TRANSPARENT, width, height).texture_id(ctx),
        };
        ui.image(texture_id, [width as f32, height as f32]);
    }

    fn card_from_release(&mut self, release: &Release) -> ReleaseCard {
        ReleaseCard {
            release: release.clone(),
            image: self.retained_images.get(release.art.first().unwrap(), 200, 200),
            player: self.player.clone(),
        }
    }
}

