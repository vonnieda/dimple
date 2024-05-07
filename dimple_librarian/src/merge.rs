use dimple_core::model::{Artist, Recording, RecordingSource, Release, ReleaseGroup};

pub trait Merge<T> {
    /// Commutative: A v B = B v A
    /// Associative: (A v B) v C = A v (B v C)
    /// Idempotent : A v A = A
    fn merge(l: T, r: T) -> T;
    fn mergability(l: &T, r: &T) -> f32;
}

// TODO leaning towards making all these use longer instead of or, which will
// help with a move towards CRDT.
// What if merge just returns an option?
// Oh, one thing I was thinking about was equality, so we could de-dupe that way,
// and I was also thinking about clustering in the file library. I think that's a
// dead end because I need clustering for s3 and other systems, too. 
// Do I? Or is S3 just a specialization of file, but otherwise different from the
// rest.
impl Merge<Self> for Artist {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            // source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            name: a.name.or(b.name),
            summary: a.summary.or(b.summary),
            country: a.country.or(b.country),
        }
    }

    fn mergability(l: &Self, r: &Self) -> f32 {
        if let (Some(l), Some(r)) = (l.key, r.key) {
            if l == r {
                return 1.0
            }
        }

        // TODO I think we want to check for all clashing KnownIds, but to do
        // that I either need to hand cod ethem all cause enum, or change that
        // enum into a struct and give it a type.
        if let (Some(l), Some(r)) = (l.mbid(), r.mbid()) {
            if l == r {
                return 1.0
            }
            else {
                return 0.0
            }
        }

        if !l.known_ids.is_disjoint(&r.known_ids) {
            return 1.0
        }

        if !l.source_ids.is_disjoint(&r.source_ids) {
            return 1.0
        }

        (unicode_fuzzy(&l.name, &r.name)
            + unicode_fuzzy(&l.disambiguation, &r.disambiguation)
            + unicode_fuzzy(&l.country, &r.country)) / 3.0
    }
}

// impl Merge<Self> for ReleaseGroup {
//     fn merge(a: Self, b: Self) -> Self {
//         Self {
//             disambiguation: a.disambiguation.or(b.disambiguation),
//             key: a.key.or(b.key),
//             known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
//             source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
//             links: a.links.union(&b.links).cloned().collect(),
//             title: a.title.or(b.title),
//             summary: a.summary.or(b.summary),

//             first_release_date: a.first_release_date.or(b.first_release_date),
//             primary_type: a.primary_type.or(b.primary_type),
//         }
//     }
    
//     fn mergability(l: &Self, r: &Self) -> f32 {
//         if let (Some(l), Some(r)) = (l.key(), r.key()) {
//             if l == r {
//                 return 1.0
//             }
//         }

//         // TODO I think we want to check for all clashing KnownIds, but to do
//         // that I either need to hand cod ethem all cause enum, or change that
//         // enum into a struct and give it a type.
//         if let (Some(l), Some(r)) = (l.mbid(), r.mbid()) {
//             if l == r {
//                 return 1.0
//             }
//             else {
//                 return 0.0
//             }
//         }

//         if !l.known_ids.is_disjoint(&r.known_ids) {
//             return 1.0
//         }

//         if !l.source_ids.is_disjoint(&r.source_ids) {
//             return 1.0
//         }

//         (unicode_fuzzy(&l.title, &r.title)
//             + unicode_fuzzy(&l.disambiguation, &r.disambiguation)) / 2.0
//     }
// }

// impl Merge<Self> for Release {
//     fn merge(a: Self, b: Self) -> Self {
//         Self {
//             disambiguation: a.disambiguation.or(b.disambiguation),
//             key: a.key.or(b.key),
//             known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
//             source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
//             links: a.links.union(&b.links).cloned().collect(),
//             title: a.title.or(b.title),
//             summary: a.summary.or(b.summary),

//             barcode: a.barcode.or(b.barcode),
//             country: a.country.or(b.country),
//             date: a.date.or(b.date),
//             packaging: a.packaging.or(b.packaging),
//             status: a.status.or(b.status),
//         }
//     }
    
//     fn mergability(l: &Self, r: &Self) -> f32 {
//         if let (Some(l), Some(r)) = (l.key(), r.key()) {
//             if l == r {
//                 return 1.0
//             }
//         }

//         // TODO I think we want to check for all clashing KnownIds, but to do
//         // that I either need to hand cod ethem all cause enum, or change that
//         // enum into a struct and give it a type.
//         if let (Some(l), Some(r)) = (l.mbid(), r.mbid()) {
//             if l == r {
//                 return 1.0
//             }
//             else {
//                 return 0.0
//             }
//         }

//         if !l.known_ids.is_disjoint(&r.known_ids) {
//             return 1.0
//         }

//         if !l.source_ids.is_disjoint(&r.source_ids) {
//             return 1.0
//         }

//         (unicode_fuzzy(&l.title, &r.title)
//             + unicode_fuzzy(&l.disambiguation, &r.disambiguation)) / 2.0
//     }
// }

// impl Merge<Self> for Recording {
//     fn merge(a: Self, b: Self) -> Self {
//         Self {
//             disambiguation: a.disambiguation.or(b.disambiguation),
//             key: a.key.or(b.key),
//             known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
//             source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
//             links: a.links.union(&b.links).cloned().collect(),
//             title: a.title.or(b.title),
//             summary: a.summary.or(b.summary),

//             annotation: a.annotation.or(b.annotation),
//             isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
//             length: a.length.or(b.length)
//         }
//     }
    
//     fn mergability(l: &Self, r: &Self) -> f32 {
//         todo!()
//     }    
// }

// impl Merge<Self> for RecordingSource {
//     fn merge(a: Self, b: Self) -> Self {
//         Self {
//             // disambiguation: a.disambiguation.or(b.disambiguation),
//             key: a.key.or(b.key),
//             known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
//             source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
//             format: a.format.or(b.format),
//             extension: a.extension.or(b.extension),
//             // links: a.links.union(&b.links).cloned().collect(),
//             // title: a.title.or(b.title),
//             // summary: a.summary.or(b.summary),

//             // annotation: a.annotation.or(b.annotation),
//             // isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
//             // length: a.length.or(b.length)
//         }
//     }
    
//     fn mergability(l: &Self, r: &Self) -> f32 {
//         todo!()
//     }
// }


fn unicode_fuzzy(l: &Option<String>, r: &Option<String>) -> f32 {
    use unicode_normalization::UnicodeNormalization;
    if let (Some(l), Some(r)) = (l, r) {
        let l = l.to_string().nfkd().to_string().to_uppercase();
        let r = r.to_string().nfkd().to_string().to_uppercase();
        strsim::sorensen_dice(&l, &r) as f32
    }
    else {
        1.0
    }
}

