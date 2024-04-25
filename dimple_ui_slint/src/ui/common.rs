use core::num;

use dimple_core::db::Db;
use dimple_core::model::Artist;
use dimple_core::model::Entity;
use dimple_core::model::Medium;
use dimple_core::model::Picture;
use dimple_core::model::Recording;
use dimple_core::model::RecordingSource;
use dimple_core::model::Release;
use dimple_core::model::ReleaseGroup;
use dimple_core::model::Track;
use image::DynamicImage;
use crate::ui::CardAdapter;
use crate::ui::ImageLinkAdapter;
use crate::ui::LinkAdapter;

use super::images::gen_fuzzy_circles;
use super::images::gen_fuzzy_rects;

impl From<Artist> for CardAdapter {
    fn from(value: Artist) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.name.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: value.disambiguation.clone().unwrap_or_default().into(),
                url: format!("dimple://artist/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}

impl From<ReleaseGroup> for CardAdapter {
    fn from(value: ReleaseGroup) -> Self {
        CardAdapter {
            image: ImageLinkAdapter {
                image: Default::default(),
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://release_group/{}", value.key.clone().unwrap_or_default()).into(),
            },
            title: LinkAdapter {
                name: value.title.clone().unwrap_or_default().into(),
                url: format!("dimple://release_group/{}", value.key.clone().unwrap_or_default()).into(),
            },
            sub_title: LinkAdapter {
                name: value.disambiguation.clone().unwrap_or_default().into(),
                url: format!("dimple://release_group/{}", value.key.clone().unwrap_or_default()).into(),
            },
        }
    }
}


// TODO hastily written bunch of adapters, clean em up
// fn link_adapters(links: Vec<Link>) -> ModelRc<LinkAdapter> {
//     let links: Vec<_> = links.into_iter().map(Into::into).collect();
//     ModelRc::from(links.as_slice())
// }

// fn entity_cards(entities: Vec<Entities>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| entity_card(ent, width, height, lib))
//         .collect()
// }

// fn entity_card(entity: &Entities, width: u32, height: u32, lib: &Librarian) -> Card {
//     match entity {
//         Entities::Artist(e) => artist_card(e, width, height, lib),
//         Entities::ReleaseGroup(e) => release_group_card(e, width, height, lib),
//         Entities::Release(e) => release_card(e, width, height, lib),
//         Entities::Recording(e) => recording_card(e, width, height, lib),
//         _ => todo!(),
//     }
// }

// fn artist_cards(entities: Vec<Artist>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| artist_card(ent, width, height, lib))
//         .collect()
// }

// fn artist_card(artist: &Artist, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Artist(artist.clone()), width, height),
//             link: Link {
//                 name: artist.name.clone().unwrap_or_default(),
//                 url: format!("dimple://artist/{}", artist.key.str()),
//             },
//         },
//         title: Link {
//             name: artist.name.clone().unwrap_or_default(),
//             url: format!("dimple://artist/{}", artist.key.str()),
//         },
//         sub_title: Link {
//             name: artist.disambiguation.clone().unwrap_or_default(),
//             url: format!("dimple://artist/{}", artist.key.str()),
//         },
//     }
// }

// fn release_group_cards(entities: Vec<ReleaseGroup>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| release_group_card(ent, width, height, lib))
//         .collect()
// }

// fn release_cards(entities: Vec<Release>, lib: &Librarian, width: u32, height: u32) -> Vec<Card> {
//     entities.par_iter()
//         .map(|ent| release_card(ent, width, height, lib))
//         .collect()
// }

// fn release_group_card(release_group: &ReleaseGroup, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::ReleaseGroup(release_group.clone()), width, height),
//             link: Link {
//                 name: release_group.title.str(),
//                 url: format!("dimple://release-group/{}", release_group.key.str()),
//             },
//         },
//         title: Link {
//             name: release_group.title.str(),
//             url: format!("dimple://release-group/{}", release_group.key.str()),
//         },
//         sub_title: Link { 
//             name: format!("{:.4} {}", release_group.first_release_date.str(), release_group.primary_type.str()),
//             url: format!("dimple://release-group/{}", release_group.key.str()),
//         },
//     }
// }

// fn release_card(release: &Release, width: u32, height: u32, lib: &Librarian) -> Card {
//     // TODO want to include disambiguation as the title, but also country,
//     // label, and date?
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Release(release.clone()), width, height),
//             link: Link {
//                 name: release.title.str(),
//                 url: format!("dimple://release/{}", release.key.str()),
//             },
//         },
//         title: Link {
//             name: release.disambiguation.clone().str(),
//             url: format!("dimple://release/{}", release.key.str()),
//         },
//         sub_title: Link { 
//             name: format!("{} {}", release.date.str(), release.country.str()),
//             url: format!("dimple://release/{}", release.key.str()),
//         },
//     }
// }

// fn recording_card(recording: &Recording, width: u32, height: u32, lib: &Librarian) -> Card {
//     Card {
//         image: ImageLink {
//             image: lib.thumbnail(&Entities::Recording(recording.clone()), width, height),
//             link: Link {
//                 name: recording.title.str(),
//                 url: format!("dimple://recording/{}", recording.key.str()),
//             },
//         },
//         title: Link {
//             name: recording.title.str(),
//             url: format!("dimple://release/{}", recording.key.str()),
//         },
//         // TODO
//         // sub_title: 
//         ..Default::default()
//     }
// }

// fn card_adapter(card: &Card) -> CardAdapter {
//     CardAdapter {
//         image: ImageLinkAdapter {
//             // TODO maybe cache, not sure the cost of this.
//             image: dynamic_image_to_slint_image(&card.image.image),
//             name: card.image.link.name.to_owned().into(),
//             url: card.image.link.url.to_owned().into(),
//         },
//         title: card.title.clone().into(),
//         sub_title: card.sub_title.clone().into(),
//     }
// }

// fn artist_links(artist: &Artist) -> Vec<Link> {
//     artist.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
//             url: format!("https://musicbrainz.org/artist/{}", artist.mbid().str()),
//         }))
//         .collect()
// }

// fn release_group_links(release_group: &ReleaseGroup) -> Vec<Link> {
//     release_group.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
//             url: format!("https://musicbrainz.org/release-group/{}", release_group.mbid().str()),
//         }))
//         .collect()
// }

// fn release_links(release: &Release) -> Vec<Link> {
//     release.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
//             url: format!("https://musicbrainz.org/release/{}", release.mbid().str()),
//         }))
//         .collect()
// }

// fn recording_links(recording: &Recording) -> Vec<Link> {
//     recording.links
//         .iter()
//         .map(|url| Link {
//             name: url.clone(),
//             url: url.clone(),
//         })
//         .chain(std::iter::once(Link { 
//             name: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
//             url: format!("https://musicbrainz.org/recording/{}", recording.mbid().str()),
//         }))
//         .collect()
// }

// fn card_adapters(cards: Vec<Card>) -> ModelRc<CardAdapter> {
//     let card_models: Vec<_>  = cards.iter()
//         .map(card_adapter)
//         .collect();
//     ModelRc::from(card_models.as_slice())
// }

// fn recording_adapters(recordings: Vec<Recording>) -> ModelRc<TrackAdapter> {
//     let adapters: Vec<_> = recordings.iter()
//         .map(|r| TrackAdapter {
//             title: LinkAdapter {
//                 name: r.title.str(),
//                 url: format!("dimple://recording/{}", r.key.str()).into(),
//             },
//             // track_number: t.number.clone().into(),
//             // length: length_to_string(t.length).into(),
//             artists: Default::default(),
//             plays: 0,
//             ..Default::default()
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// fn track_adapters(tracks: Vec<Track>) -> ModelRc<TrackAdapter> {
//     let adapters: Vec<_> = tracks.iter()
//         .map(|t| TrackAdapter {
//             title: LinkAdapter {
//                 name: t.title.clone().into(),
//                 url: format!("dimple://recording/{}", t.recording.key.str()).into(),
//             },
//             track_number: t.number.clone().into(),
//             length: length_to_string(t.length).into(),
//             artists: Default::default(),
//             plays: 0,
//             source_count: t.sources.len() as i32,
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// fn media_adapters(media: Vec<Medium>) -> ModelRc<MediumAdapter> {
//     let adapters: Vec<_> = media.iter()
//         .map(|m| MediumAdapter {
//             title: format!("{} {} of {}", m.format, m.position, m.disc_count).into(),
//             tracks: track_adapters(m.tracks.clone()),
//         })
//         .collect();
//     ModelRc::from(adapters.as_slice())
// }

// fn dynamic_image_to_slint_image(dynamic_image: &DynamicImage) -> slint::Image {
//     let rgba8_image = dynamic_image.clone().into_rgba8();
//     let shared_pixbuf = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
//         rgba8_image.as_raw(),
//         rgba8_image.width(),
//         rgba8_image.height(),
//     );
//     slint::Image::from_rgba8(shared_pixbuf)
// }

// Creates a simple score for a release to use when selecting a
// a default release.
// TODO this is super naive, just needed something to set the example.
// fn score_release(r: &Release) -> f64 {
//     let mut score = 0.;
//     let country = r.country.str().to_lowercase();
//     if country == "xw" {
//         score += 1.0;
//     }                
//     else if country == "us" || country == "gb" || country == "xe" {
//         score += 0.7;
//     }
//     else if !country.is_empty() {
//         score += 0.1;
//     }

//     if r.status.str().to_lowercase() == "official" {
//         score += 1.0;
//     }

//     let packaging = r.packaging.str().to_lowercase();
//     if packaging == "digipak" {
//         score += 1.0;
//     }
//     else if packaging == "jewelcase" {
//         score += 0.5;
//     }

//     // if !r.media.is_empty() {
//     //     let mut media_format_score = 0.;
//     //     for media in r.media.clone() {
//     //         let format = media.format.to_lowercase();
//     //         if format == "digital media" {
//     //             media_format_score += 1.0;
//     //         }
//     //         else if format == "cd" {
//     //             media_format_score += 0.5;
//     //         }
//     //     }
//     //     score += media_format_score / r.media.len() as f64;
//     // }

//     score / 4.
// }

// #[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
// struct Link {
//     name: String,
//     url: String,
// }

// #[derive(Clone, Debug, PartialEq, Default)]
// struct ImageLink {
//     link: Link,
//     image: DynamicImage,
// }

// #[derive(Clone, Debug, PartialEq, Default)]
// struct Card {
//     image: ImageLink,
//     title: Link,
//     sub_title: Link,
// }

// impl From<Link> for LinkAdapter {
//     fn from(value: Link) -> Self {
//         Self {
//             name: value.name.into(),
//             url: value.url.into(),
//         }
//     }
// }

// trait OptStr {
//     fn str(&self) -> String;
// }

// impl OptStr for Option<String> {
//     fn str(&self) -> String {
//         self.clone().unwrap_or_default()
//     }
// }

// trait AsSharedString {
//     fn shared(&self) -> SharedString; 
// }

// impl AsSharedString for Option<String> {
//     fn shared(&self) -> SharedString {
//         SharedString::from(self.str())        
//     }
// }

// impl AsSharedString for &str {
//     fn shared(&self) -> SharedString {
//         SharedString::from(self.to_string())
//     }
// }

// impl AsSharedString for String {
//     fn shared(&self) -> SharedString {
//         SharedString::from(self)        
//     }
// }

// trait AsListViewItem {
//     fn listview_item(&self) -> StandardListViewItem;
// }

// impl AsListViewItem for Option<String> {
//     fn listview_item(&self) -> StandardListViewItem {
//         StandardListViewItem::from(self.shared())
//     }
// }

// impl AsListViewItem for &str {
//     fn listview_item(&self) -> StandardListViewItem {
//         StandardListViewItem::from(self.shared())
//     }
// }

// impl AsListViewItem for String {
//     fn listview_item(&self) -> StandardListViewItem {
//         StandardListViewItem::from(self.shared())
//     }
// }




// {
//     let librarian = librarian.clone();
//     let ui = ui.clone();
//     std::thread::spawn(move || {
//         thread::sleep(Duration::from_secs(5));
//         log::info!("here we go!");
//         ui.upgrade_in_event_loop(move |ui| {
//             let adapter = ui.get_artist_list();
//             // let pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(640, 480);
//             // let image = Image::from_rgba8_premultiplied(pixel_buffer);

//             let mut demo_image = image::open("images/light.png").expect("Error loading demo image").into_rgba8();

//             image::imageops::colorops::brighten_in_place(&mut demo_image, 20);
            
//             let buffer = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
//                 demo_image.as_raw(),
//                 demo_image.width(),
//                 demo_image.height(),
//             );
//             let image = Image::from_rgba8(buffer);

//             for (i, card) in adapter.cards.iter().enumerate() {
//                 let mut card = card.clone();
//                 card.image.image = image.clone();
//                 card.title.name = "Wow".to_string().into();
//                 adapter.cards.set_row_data(i, card);
//             }
//         })
//         .unwrap();
//     });
// }

// fn create_artist(db: &dyn Db) -> Artist {
//     let artist = Artist {
//         name: Some(fakeit::name::full()),
//         summary: Some(fakeit::hipster::paragraph(1, 4, 40, " ".to_string())),
//         country: Some(fakeit::address::country_abr()),
//         disambiguation: Some(fakeit::address::country()),
//         ..Default::default()
//     };
//     let artist_pic = Picture::new(&gen_fuzzy_circles(1000, 1000));

//     let artist: Artist = db.insert(&artist.model()).unwrap().into();
//     let artist_pic: Picture = db.insert(&artist_pic.model()).unwrap().into();

//     db.link(&artist_pic.model(), &artist.model());

//     artist
// }

// fn create

// fn create_picture(db: &dyn Db, image: &DynamicImage) -> Picture {
//     let picture = Picture::new(image);
//     db.insert(&picture.model()).unwrap().into()
// }

// fn create_artist(db: &dyn Db) -> Artist {
//     let artist = Artist {
//         name: Some(fakeit::name::full()),
//         summary: Some(fakeit::hipster::paragraph(1, 4, 40, " ".to_string())),
//         country: Some(fakeit::address::country_abr()),
//         disambiguation: Some(fakeit::address::country()),
//         ..Default::default()
//     };
//     let artist: Artist = db.insert(&artist.model()).unwrap().into();
//     let artist_pic = create_picture(db, &gen_fuzzy_circles(1000, 1000));
//     db.link(&artist_pic.model(), &artist.model());

//     artist
// }

// fn create_release_group(db: &dyn Db) -> ReleaseGroup {
//     let 
// }

pub fn create_artist(db: &dyn Db) -> Artist {
    let artist = db.insert(&Artist {
        name: Some(fakeit::name::full()),
        summary: Some(fakeit::hipster::paragraph(1, 4, 40, " ".to_string())),
        country: Some(fakeit::address::country_abr()),
        disambiguation: Some(fakeit::address::country()),
        ..Default::default()
    }.model()).unwrap();
    let artist_pic = db.insert(&Picture::new(&gen_fuzzy_circles(1000, 1000)).model()).unwrap();
    db.link(&artist_pic, &artist).unwrap();
    artist.into()
}

pub fn create_release_group(db: &dyn Db) -> ReleaseGroup {
    let release_group = db.insert(&ReleaseGroup {
        title: Some(fakeit::hipster::sentence(2)),
        ..Default::default()
    }.model()).unwrap();
    let release_group_pic = db.insert(&Picture::new(&gen_fuzzy_rects(1000, 1000)).model()).unwrap();
    db.link(&release_group_pic, &release_group).unwrap();
    release_group.into()
}

pub fn create_random_data(db: &dyn Db, num_artists: u32) {
    // let range = 0..num_artists;
    for _ in 0..num_artists {
        let artist = create_artist(db);
        for _ in 0..fakeit::misc::random(0, 7) {
            let release_group = create_release_group(db);
            db.link(&release_group.model(), &artist.model()).unwrap();
        }
    }
}
