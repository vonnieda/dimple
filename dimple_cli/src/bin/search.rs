use musicbrainz_rs::entity::artist::{Artist, ArtistSearchQuery};
use musicbrainz_rs::prelude::*;

fn main() {
    // let query = ArtistSearchQuery::query_builder()
    // .artist("Opeth")
    // // .and()
    // // .country("US")
    // .build();

    // let query_result = Artist::search(query)
    //     .execute()
    //     .unwrap();
    // dbg!(&query_result);

    musicbrainz_rs::config::set_user_agent("DimpleMusicPlayer ( jason@vonnieda.org )");

    let opeth = Artist::fetch()
        .id("c14b4180-dc87-481e-b17a-64e4150f90f6")
        .with_releases()
        .execute();

    dbg!(opeth);
}