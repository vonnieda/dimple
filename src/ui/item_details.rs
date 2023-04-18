use std::{sync::{Arc, RwLock}};

use eframe::{egui::{Ui, Grid}};

use crate::{music_library::{Artist, Genre, Release, Track, Playlist}, player::{PlayerHandle, Player}, librarian::Librarian};

use super::{card_grid::{LibraryItem, CardGrid, Card}, theme::Theme};

// TODO feels like it's time to split this up
pub struct ItemDetails {
    player: PlayerHandle,
    librarian: Arc<Librarian>,
}

impl ItemDetails {
    pub fn new(player: PlayerHandle, librarian: Arc<Librarian>) -> Self {
        Self {
            player,
            librarian,
        }
    }

    pub fn ui(&mut self, item: LibraryItem, ui: &mut Ui) -> Option<LibraryItem> {
        use LibraryItem::*;

        match item {
            Release(release) => self.release(&release, ui),
            Artist(artist) => self.artist(&artist, ui),
            Genre(genre) => self.genre(&genre, ui),
            Playlist(playlist) => self.playlist(&playlist, ui),
            Track(track) => self.track(&track, ui),
            Player(player) => self.now_playing(player, ui),
        }
    }

    pub fn now_playing(&mut self, player: Arc<RwLock<Player>>, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        // TODO generate cool artwork for the playlist
                        let mut art = vec![];
                        if let Some(item) = player.read().unwrap().current_queue_item() {
                            art = item.release.art;
                        }
                        theme.carousel(&art, 250, ui);
                        // self.play_controls(&LibraryItem::Release(release.clone()), ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Now Playing");
                    });
                    ui.horizontal(|ui| {
                        let mut artists = Vec::new();
                        for queue_item in player.read().unwrap().queue().iter() {
                            for artist in &queue_item.release.artists {
                                if !artists.contains(artist) {
                                    artists.push(artist.clone());
                                }
                            }
                        }
                        ui.label("by");
                        if let Some(item) = self.artist_links(&artists, ui) {
                            action = Some(item);
                        }
                    });
                    ui.horizontal(|ui| {
                        let mut genres = Vec::new();
                        for queue_item in player.read().unwrap().queue().iter() {
                            for genre in &queue_item.release.genres {
                                if !genres.contains(genre) {
                                    genres.push(genre.clone());
                                }
                            }
                        }
                        ui.label("in");
                        if let Some(item) = self.genre_links(&genres, ui) {
                            action = Some(item);
                        }
                    });
                });
            });
            Grid::new("tracks").show(ui, |ui| {
                for (i, queue_item) in player.read().unwrap().queue().iter().enumerate() {
                    ui.label(&i.to_string());
                    ui.label(&queue_item.track.title);
                    ui.label(&queue_item.release.title);
                    ui.end_row();
                }
            });
        });
        action
    }

    pub fn release(&mut self, release: &Release, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        theme.carousel(&release.art, 250, ui);
                        self.play_controls(&LibraryItem::Release(release.clone()), ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&release.title);
                    });
                    ui.horizontal(|ui| {
                        ui.label("by");
                        if let Some(item) = self.artist_links(&release.artists, ui) {
                            action = Some(item);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("in");
                        if let Some(item) = self.genre_links(&release.genres, ui) {
                            action = Some(item);
                        }
                    });
                });
            });
        });
        action
    }

    pub fn artist(&mut self, artist: &Artist, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        theme.carousel(&artist.art, 250, ui);
                        self.play_controls(&LibraryItem::Artist(artist.clone()), ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&artist.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("in");
                        self.genre_links(&artist.genres, ui);
                    });
                })
            });
            let releases = self.librarian.releases_by_artist(&artist);
            let cards: Vec<Box<dyn Card>> = releases.into_iter().map(|release| Box::new(release) as Box<dyn Card>).collect();
            if let Some(item) = CardGrid::default().ui(cards.as_slice(), 100.0, 100.0, ui) {
                action = Some(item);
            }
        });
        action
    }

    pub fn genre(&mut self, genre: &Genre, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        theme.carousel(&genre.art, 250, ui);
                        self.play_controls(&LibraryItem::Genre(genre.clone()), ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&genre.name);
                    });
                })
            });
            let releases = self.librarian.releases_by_genre(&genre);
            let cards: Vec<Box<dyn Card>> = releases.into_iter().map(|release| Box::new(release) as Box<dyn Card>).collect();
            if let Some(item) = CardGrid::default().ui(&cards, 100.0, 100.0, ui) {
                action = Some(item);
            }
        });
        action
    }

    pub fn playlist(&mut self, playlist: &Playlist, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        theme.carousel(&playlist.art, 250, ui);
                        self.play_controls(&LibraryItem::Playlist(playlist.clone()), ui);
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

    pub fn track(&mut self, track: &Track, ui: &mut Ui) -> Option<LibraryItem> {
        let theme = Theme::get(ui.ctx());
        let mut action = None;
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        theme.carousel(&track.art, 250, ui);
                        self.play_controls(&LibraryItem::Track(track.clone()), ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading(&track.title);
                    });
                    ui.horizontal(|ui| {
                        ui.label("by");
                        if let Some(item) = self.artist_links(&track.artists, ui) {
                            action = Some(item);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("in");
                        self.genre_links(&track.genres, ui);
                    });
                })
            })
        });
        action
    }

    pub fn play_controls(&self, library_item: &LibraryItem, ui: &mut Ui) {
        let theme = Theme::get(ui.ctx());
        if Theme::icon_button(&theme.add_icon, 48, 48, ui).clicked() {
            match library_item {
                LibraryItem::Release(release) => {
                    self.player.write().unwrap().queue_release(release);
                },
                LibraryItem::Artist(_) => todo!(),
                LibraryItem::Genre(_) => todo!(),
                LibraryItem::Playlist(_) => todo!(),
                LibraryItem::Track(_track) => todo!(),
                LibraryItem::Player(_player) => todo!(),
            }
        }
    }

    pub fn artist_links(&self, artists: &Vec<Artist>, ui: &mut Ui) -> Option<LibraryItem> {
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
                    ui.label(", ");
                }
            }
        });
        action
    }

    pub fn genre_links(&self, genres: &[Genre], ui: &mut Ui) -> Option<LibraryItem> {
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
                    ui.label(", ");
                }
            }
        });
        action
    }
}



// Links for artist(s), release, genre(s)
// So, for each kind of thing, what do we show?
// 
// Release(release)
//     Vertical
//         Horizontal
//             Carousel(art, 300, 300)
//             Vertical
//                 Horizontal
//                     ImageButton(release_icon)
//                     Heading(release.title)
//                 Horizontal
//                     ImageButton(artist_icon)
//                     Links(release.artists)
//                 Horizontal
//                     ImageButton(genre_icon)
//                     Links(release.genres)
//         CardGrid(more_like_this)
//     
// Artist: Art(Carousel), Name, Genre(s), Grid(Releases), Grid(More Artists Like This)
// Track: Art(Carousel), Title, Lyrics
