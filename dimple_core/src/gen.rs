use std::collections::HashSet;
use rand::seq::{IndexedRandom as _, SliceRandom};
use rand::thread_rng;

use crate::library::Library;
use crate::model::{Artist, PlaylistItem};

#[derive(Debug)]
struct LocalTrack {
    name: String,
    genres: HashSet<String>,
}

#[derive(Debug)]
struct LocalArtist {
    name: String,
    genres: HashSet<String>,
    tracks: Vec<LocalTrack>,
}

fn jaccard_similarity(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    let intersection = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 { 0.0 } else { intersection / union }
}

pub fn generate_artist_playlist(library: &Library, artist: &Artist) 
    -> anyhow::Result<Vec<PlaylistItem>> {
    Ok(vec![])
}

fn generate_playlist(
    seed: &LocalArtist,
    artists: &[LocalArtist],
    top_n_artists: usize,
    tracks_per_artist: usize,
    alpha: f32, // weight for artist similarity
) -> Vec<String> {
    let mut rng = thread_rng();

    // Step 1: Compute artist similarities
    let mut artist_sims: Vec<(&LocalArtist, f32)> = artists
        .iter()
        .filter(|a| a.name != seed.name)
        .map(|a| (a, jaccard_similarity(&seed.genres, &a.genres)))
        .filter(|(_, sim)| *sim > 0.0)
        .collect();

    artist_sims.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    artist_sims.truncate(top_n_artists);

    // Step 2: Score tracks using weighted formula
    let mut scored_tracks = Vec::new();
    for (artist, artist_sim) in artist_sims {
        for track in &artist.tracks {
            let track_sim = jaccard_similarity(&seed.genres, &track.genres);
            let score = alpha * artist_sim + (1.0 - alpha) * track_sim;
            scored_tracks.push((track, artist, score));
        }
    }

    // Step 3: Sort tracks by score and pick top per artist
    scored_tracks.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    let mut playlist = Vec::new();
    let mut artist_track_count = std::collections::HashMap::new();

    for (track, artist, score) in scored_tracks {
        let count = artist_track_count.entry(artist.name.clone()).or_insert(0);
        if *count < tracks_per_artist {
            playlist.push(format!("{} – {} (score: {:.2})", artist.name, track.name, score));
            *count += 1;
        }
    }

    playlist.shuffle(&mut rng);
    playlist
}

#[test]
fn test_generate_artist_playlist() {
    
}

#[test]
pub fn test_generate_playlist() {
    let artists = vec![
        LocalArtist {
            name: "Radiohead".into(),
            genres: ["rock", "alternative", "experimental"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Paranoid Android".into(), genres: ["rock", "experimental"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Karma Police".into(), genres: ["rock", "alternative"].into_iter().map(String::from).collect() },
                LocalTrack { name: "No Surprises".into(), genres: ["alternative", "experimental"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Coldplay".into(),
            genres: ["rock", "pop", "alternative"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Yellow".into(), genres: ["rock", "pop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Fix You".into(), genres: ["rock", "alternative"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Clocks".into(), genres: ["pop", "alternative"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Aphex Twin".into(),
            genres: ["electronic", "experimental", "ambient"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Windowlicker".into(), genres: ["electronic", "experimental"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Come to Daddy".into(), genres: ["electronic", "experimental"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Alberto Balsalm".into(), genres: ["ambient", "electronic"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Kendrick Lamar".into(),
            genres: ["hip-hop", "rap", "conscious"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "HUMBLE.".into(), genres: ["hip-hop", "rap"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Alright".into(), genres: ["hip-hop", "conscious"].into_iter().map(String::from).collect() },
                LocalTrack { name: "DNA.".into(), genres: ["rap", "hip-hop"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Mozart".into(),
            genres: ["classical", "baroque", "orchestral"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Requiem in D Minor".into(), genres: ["classical", "orchestral"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Eine kleine Nachtmusik".into(), genres: ["classical", "baroque"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Symphony No. 40".into(), genres: ["orchestral", "classical"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Daft Punk".into(),
            genres: ["electronic", "house", "dance"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "One More Time".into(), genres: ["house", "dance"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Around the World".into(), genres: ["electronic", "house"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Get Lucky".into(), genres: ["dance", "electronic"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Johnny Cash".into(),
            genres: ["country", "folk", "americana"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Folsom Prison Blues".into(), genres: ["country", "folk"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Ring of Fire".into(), genres: ["country", "americana"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Hurt".into(), genres: ["folk", "americana"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Metallica".into(),
            genres: ["metal", "thrash", "rock"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Master of Puppets".into(), genres: ["thrash", "metal"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Enter Sandman".into(), genres: ["metal", "rock"].into_iter().map(String::from).collect() },
                LocalTrack { name: "One".into(), genres: ["metal", "thrash"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Billie Eilish".into(),
            genres: ["pop", "electropop", "alternative"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "bad guy".into(), genres: ["pop", "electropop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "everything i wanted".into(), genres: ["alternative", "pop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "bury a friend".into(), genres: ["electropop", "alternative"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Miles Davis".into(),
            genres: ["jazz", "bebop", "fusion"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "So What".into(), genres: ["jazz", "bebop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "All Blues".into(), genres: ["jazz", "fusion"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Freddie Freeloader".into(), genres: ["bebop", "jazz"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "The Smiths".into(),
            genres: ["indie", "alternative", "rock"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "There Is a Light That Never Goes Out".into(), genres: ["indie", "alternative"].into_iter().map(String::from).collect() },
                LocalTrack { name: "This Charming Man".into(), genres: ["indie", "rock"].into_iter().map(String::from).collect() },
                LocalTrack { name: "How Soon Is Now?".into(), genres: ["alternative", "rock"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Deadmau5".into(),
            genres: ["electronic", "progressive-house", "edm"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Strobe".into(), genres: ["progressive-house", "electronic"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Ghosts 'n' Stuff".into(), genres: ["edm", "electronic"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Some Chords".into(), genres: ["progressive-house", "edm"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Dolly Parton".into(),
            genres: ["country", "pop", "folk"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Jolene".into(), genres: ["country", "folk"].into_iter().map(String::from).collect() },
                LocalTrack { name: "9 to 5".into(), genres: ["country", "pop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "I Will Always Love You".into(), genres: ["pop", "country"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Slayer".into(),
            genres: ["metal", "thrash", "extreme"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Raining Blood".into(), genres: ["thrash", "extreme"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Angel of Death".into(), genres: ["metal", "thrash"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Seasons in the Abyss".into(), genres: ["metal", "extreme"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Bob Marley".into(),
            genres: ["reggae", "ska", "roots"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "No Woman, No Cry".into(), genres: ["reggae", "roots"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Redemption Song".into(), genres: ["reggae", "folk"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Three Little Birds".into(), genres: ["reggae", "ska"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Björk".into(),
            genres: ["experimental", "art-pop", "electronic"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Jóga".into(), genres: ["experimental", "art-pop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Hyperballad".into(), genres: ["electronic", "experimental"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Army of Me".into(), genres: ["art-pop", "electronic"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "AC/DC".into(),
            genres: ["rock", "hard-rock", "blues-rock"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Back in Black".into(), genres: ["hard-rock", "rock"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Highway to Hell".into(), genres: ["rock", "blues-rock"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Thunderstruck".into(), genres: ["hard-rock", "blues-rock"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Ariana Grande".into(),
            genres: ["pop", "r&b", "dance-pop"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "thank u, next".into(), genres: ["pop", "r&b"].into_iter().map(String::from).collect() },
                LocalTrack { name: "7 rings".into(), genres: ["pop", "dance-pop"].into_iter().map(String::from).collect() },
                LocalTrack { name: "positions".into(), genres: ["r&b", "pop"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Sonic Youth".into(),
            genres: ["noise", "experimental", "alternative"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Teen Age Riot".into(), genres: ["noise", "alternative"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Bull in the Heather".into(), genres: ["experimental", "alternative"].into_iter().map(String::from).collect() },
                LocalTrack { name: "Kool Thing".into(), genres: ["noise", "experimental"].into_iter().map(String::from).collect() },
            ],
        },
        LocalArtist {
            name: "Pavarotti".into(),
            genres: ["opera", "classical", "vocal"].into_iter().map(String::from).collect(),
            tracks: vec![
                LocalTrack { name: "Nessun Dorma".into(), genres: ["opera", "classical"].into_iter().map(String::from).collect() },
                LocalTrack { name: "La donna è mobile".into(), genres: ["opera", "vocal"].into_iter().map(String::from).collect() },
                LocalTrack { name: "O sole mio".into(), genres: ["classical", "vocal"].into_iter().map(String::from).collect() },
            ],
        },
    ];

    let mut rng = thread_rng();
    let seed = artists.choose(&mut rng).unwrap();
    let playlist = generate_playlist(seed, &artists, 10, 2, 0.6);

    println!("Weighted similarity playlist from '{}':", seed.name);
    for track in playlist {
        println!("  {}", track);
    }
}

