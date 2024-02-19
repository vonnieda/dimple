use dimple_core::{collection::Collection, model::Entities};
use dimple_sled_library::sled_library::SledLibrary;
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::librarian::Librarian;

// 1. Attempt to find by key
// 2. Attempt to find by source_id
// 3. Attempt to find by known_id
// 4. Attempt to find by fuzzy?
// 5. Give up.
pub fn find_match(librarian: &Librarian, local_library: &SledLibrary, model: &Entities) -> Option<Entities> {
    // Find by key
    if let Some(model) = local_library.get(model) {
        return Some(model)
    }

    // Find by source id, known id, or fuzzy match
    for m in local_library.list(model, None) {
        let l = model;
        let r = m;
        // Shares a source id?
        if !l.source_ids().is_disjoint(&r.source_ids()) {
            return Some(r)
        }

        // Shares a known id?
        if !l.known_ids().is_disjoint(&r.known_ids()) {
            return Some(r)
        }

        fn fuzzy(l: &Option<String>, r: &Option<String>) -> bool {
            if let (Some(l), Some(r)) = (l, r) {
                let matcher = SkimMatcherV2::default();
                matcher.fuzzy_match(r, l).is_some()
            }
            else {
                true
            }
        }

        // Fuzzy match on fields?
        match (l, r) {
            (Entities::Artist(l), Entities::Artist(r)) => {
                if fuzzy(&l.name, &r.name) 
                    && fuzzy(&l.disambiguation, &r.disambiguation)
                    && fuzzy(&l.country, &r.country) {
                    return Some(Entities::Artist(r))
                }
            },
            (Entities::ReleaseGroup(l), Entities::ReleaseGroup(r)) => {
                if fuzzy(&l.title, &r.title) 
                    && fuzzy(&l.first_release_date, &r.disambiguation) 
                    && fuzzy(&l.primary_type, &r.primary_type) {
                    return Some(Entities::ReleaseGroup(r))
                }
            },
            // TODO this either needs to go away or take parents and
            // children into consideration. It's really gonna have to
            // be a look up the parent, see if it fits, see if the parent
            // already has a similar child, etc.
            // Cause right now this is merging Spidergawd with Opeth because
            // In Caude Venenum fuzzy matches IV and there's nothing
            // else to go on.
            (Entities::Release(l), Entities::Release(r)) => {
                if fuzzy(&l.title, &r.title) 
                    && fuzzy(&l.disambiguation, &r.disambiguation)
                    && fuzzy(&l.date, &r.date)
                    && fuzzy(&l.barcode, &r.barcode)
                    && fuzzy(&l.packaging, &r.packaging)
                    && fuzzy(&l.status, &r.status)
                    && fuzzy(&l.country, &r.country) {
                    return Some(Entities::Release(r))
                }
            },
            _ => ()
        };
    }

    // Give up
    None
}
