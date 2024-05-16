pub mod db;
pub mod model;
pub const USER_AGENT: &str = "Dimple/0.0.1 +https://github.com/vonnieda/dimple +jason@vonnieda.org";

// pub struct LibrarySupport {
// }

// pub struct RequestToken {
//     library_name: String,
//     start_time: Instant,
//     url: String,
// }

// impl LibrarySupport {
//     pub fn start_request(library: &dyn Collection, url: &str) -> RequestToken {
//         RequestToken {
//             library_name: library.name().to_owned(),
//             start_time: Instant::now(),
//             url: url.to_owned(),
//         }
//     }

//     pub fn end_request(token: RequestToken, status_code: Option<u16>, length: Option<u64>) {
//         log::info!("{} [{:?}] {}ms {:?} {}", 
//             token.library_name, 
//             status_code, 
//             token.start_time.elapsed().as_millis(), 
//             length,
//             token.url);
//     }
// }