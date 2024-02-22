use std::time::Instant;

use colored::Colorize;

use crate::model::Entities;


pub trait Collection: Send + Sync {
    /// Get a user friendly display name for the Library.
    fn name(&self) -> String;

    /// Search the library for entities that match the query string. How the
    /// search query is interpreted is left up to the implementation. In
    /// general it should return, at least, matching Artists, Releases,
    /// Genres, and Tracks.
    /// TODO I think this will become model specific
    fn search(&self, _query: &str) -> Box<dyn Iterator<Item = Entities>> {
        Box::new(std::iter::empty())
    }

    /// Fetch a complete record for the given entity. The Library implementation
    /// can choose to use any appropriate fields in the entity as the selection
    /// criteria. Typically mbid or another id will be used. The Library should
    /// return only those fields that are fetched, it should not return fields
    /// that come from the source entity.
    fn fetch(&self, _entity: &Entities) -> Option<Entities> {
        None
    }

    // TODO Eventually this will allow access to more image types.
    // TODO add preferred_width, preferred_height
    fn image(&self, _entity: &Entities) -> Option<image::DynamicImage> {
        None
    }

    // TODO really only meant for RecordingSource. Maybe that's all it needs to
    // be for?
    fn stream(&self, _entity: &Entities) -> Option<Box<dyn Iterator<Item = u8>>> {
        None
    }

    /// List all of the entities of the given type, constrained by the specified
    /// related_to object, if given. This can be used to list primary entities
    /// as well as list entities that have a foreign key to an entity.
    /// 
    /// Examples:
    /// All tracks:
    /// let tracks = list(Model::Track, None);
    /// 
    /// Tracks in an album:
    /// let album: Album;
    /// let tracks = list(Model::Track, album);
    /// 
    /// Lyrics for a Track:
    /// let track: Track;
    /// let lyrics = list(Model::Lyrics, track);
    /// 
    /// Albums for an Artist:
    /// let artist: Artist;
    /// let albums = list(Model::Album, artist);
    /// 
    /// All artists:
    /// let artists = list(Model::Artist, None);
    /// 
    /// Sources for a track:
    /// let track: Track;
    /// let sources = list(Model::TrackSource, track);
    fn list(&self, _of_type: &Entities, _related_to: Option<&Entities>) -> Box<dyn Iterator<Item = Entities>> {
        Box::new(std::iter::empty())
    }
}

pub struct LibrarySupport {
}

pub struct RequestToken {
    library_name: String,
    start_time: Instant,
    url: String,
}

impl LibrarySupport {
    pub fn start_request(library: &dyn Collection, url: &str) -> RequestToken {
        RequestToken {
            library_name: library.name().to_owned(),
            start_time: Instant::now(),
            url: url.to_owned(),
        }
    }

    pub fn end_request(token: RequestToken, status_code: Option<u16>, length: Option<u64>) {
        log::info!("{} [{:?}] {}ms {:?} {}", 
            token.library_name.blue(), 
            status_code, 
            token.start_time.elapsed().as_millis(), 
            length,
            token.url.yellow());
    }
}