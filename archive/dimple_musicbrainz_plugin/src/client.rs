// https://musicbrainz.org/ws/2/release-group?fmt=json&offset=0&limit=100&artist=73084492-3e59-4b7f-aa65-572a9d7691d5
// {
//     "release-group-offset": 0,
//     "release-groups": [
//       {
//         "disambiguation": "",
//         "primary-type-id": "f529b476-6e62-324f-b0aa-1f3e33d313fc",
//         "secondary-types": [],
//         "secondary-type-ids": [],
//         "first-release-date": "2018-02-05",
//         "primary-type": "Album",
//         "title": "Lightness",
//         "id": "41e8205d-125f-4df0-bbe7-0ee5ed843199"
//       },
//       {
//         "primary-type": "EP",
//         "first-release-date": "2016-01-12",
//         "title": "We Were Heading North",
//         "id": "92e7e5d4-9183-4d72-bddb-1e052d317bef",
//         "disambiguation": "",
//         "secondary-type-ids": [],
//         "secondary-types": [],
//         "primary-type-id": "6d0c5bf6-7a33-3420-a519-44fc63eedebf"
//       }
//     ],
//     "release-group-count": 2
//   }

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct ReleaseGroups {
    pub release_group_offset: u32,
    pub release_groups: Vec<musicbrainz_rs::entity::release_group::ReleaseGroup>,
    pub release_group_count: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct Releases {
    pub release_offset: u32,
    pub releases: Vec<musicbrainz_rs::entity::release::Release>,
    pub release_count: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct ArtistResults {
    pub created: String,
    pub offset: u32,
    pub artists: Vec<musicbrainz_rs::entity::artist::Artist>,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct ReleaseGroupResults {
    pub offset: u32,
    pub release_groups: Vec<musicbrainz_rs::entity::release_group::ReleaseGroup>,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct RecordingResults {
    pub offset: u32,
    pub recordings: Vec<musicbrainz_rs::entity::recording::Recording>,
    pub count: u32,
}

