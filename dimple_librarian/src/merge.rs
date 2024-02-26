trait Merge<T> {
    // TODO should probably be references.
    fn merge(a: T, b: T) -> T;
}

impl Merge<Entities> for Entities {
    fn merge(left: Entities, right: Entities) -> Self {
        match (left, right) {
            (Entities::Artist(left), Entities::Artist(right)) => {
                Artist::merge(left, right).entity()
            },
            (Entities::ReleaseGroup(left), Entities::ReleaseGroup(right)) => {
                ReleaseGroup::merge(left, right).entity()
            },
            (Entities::Release(left), Entities::Release(right)) => {
                Release::merge(left, right).entity()
            },
            (Entities::Recording(left), Entities::Recording(right)) => {
                Recording::merge(left, right).entity()
            },
            (Entities::RecordingSource(left), Entities::RecordingSource(right)) => {
                RecordingSource::merge(left, right).entity()
            },
            _ => todo!()
        }
    }
}

// TODO leaning towards making all these use longer instead of or, which will
// help with a move towards CRDT.
impl Merge<Self> for Artist {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            name: a.name.or(b.name),
            summary: a.summary.or(b.summary),
            country: a.country.or(b.country),
        }
    }
}

impl Merge<Self> for ReleaseGroup {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            title: a.title.or(b.title),
            summary: a.summary.or(b.summary),

            first_release_date: a.first_release_date.or(b.first_release_date),
            primary_type: a.primary_type.or(b.primary_type),
        }
    }
}

impl Merge<Self> for Release {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            title: a.title.or(b.title),
            summary: a.summary.or(b.summary),


            barcode: a.barcode.or(b.barcode),
            country: a.country.or(b.country),
            date: a.date.or(b.date),
            packaging: a.packaging.or(b.packaging),
            status: a.status.or(b.status),
        }
    }
}

impl Merge<Self> for Recording {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            links: a.links.union(&b.links).cloned().collect(),
            title: a.title.or(b.title),
            summary: a.summary.or(b.summary),

            annotation: a.annotation.or(b.annotation),
            isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
            length: a.length.or(b.length)
        }
    }
}

impl Merge<Self> for RecordingSource {
    fn merge(a: Self, b: Self) -> Self {
        Self {
            // disambiguation: a.disambiguation.or(b.disambiguation),
            key: a.key.or(b.key),
            known_ids: a.known_ids.union(&b.known_ids).cloned().collect(),
            source_ids: a.source_ids.union(&b.source_ids).cloned().collect(),
            format: a.format.or(b.format),
            extension: a.extension.or(b.extension),
            // links: a.links.union(&b.links).cloned().collect(),
            // title: a.title.or(b.title),
            // summary: a.summary.or(b.summary),

            // annotation: a.annotation.or(b.annotation),
            // isrcs: a.isrcs.union(&b.isrcs).cloned().collect(),
            // length: a.length.or(b.length)
        }
    }
}
