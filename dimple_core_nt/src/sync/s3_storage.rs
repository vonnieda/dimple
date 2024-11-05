use std::env;

use s3::{creds::Credentials, Bucket, Region};

use super::storage::Storage;

pub struct S3Storage {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String, // TODO currently ignored, will add support back later
}

impl S3Storage {
    pub fn new(access_key: &str, secret_key: &str, region: &str, endpoint: &str,
        bucket: &str, prefix: &str) -> Self {

        S3Storage {
            access_key: access_key.to_owned(),
            secret_key: secret_key.to_owned(),
            region: region.to_owned(),
            endpoint: endpoint.to_owned(),
            bucket: bucket.to_owned(),
            prefix: prefix.to_owned(),
        }
    }

    fn open_bucket(&self) -> Bucket {
        let credentials = Credentials::new(Some(&self.access_key), Some(&self.secret_key), None, None, None).unwrap();
        let region = Region::Custom { 
            region: self.region.to_owned(), 
            endpoint: self.endpoint.to_owned() 
        };
        Bucket::new(
            &self.bucket,
            region,
            credentials,
        ).unwrap()
    }    

    fn strip_prefix(s: &str, prefix: &str) -> String {
        if s.starts_with(prefix) {
            return s[prefix.len()..].to_string()
        }
        s.to_string()
    }
}

impl Default for S3Storage {
    fn default() -> Self {
        // TODO make the env key prefix configuratble
        let access_key = env::var("DIMPLE_TEST_S3_ACCESS_KEY").expect("Missing DIMPLE_TEST_S3_ACCESS_KEY environment variable.");
        let secret_key = env::var("DIMPLE_TEST_S3_SECRET_KEY").expect("Missing DIMPLE_TEST_S3_SECRET_KEY environment variable.");
        let region = env::var("DIMPLE_TEST_S3_REGION").expect("Missing DIMPLE_TEST_S3_REGION environment variable.");
        let endpoint = env::var("DIMPLE_TEST_S3_ENDPOINT").expect("Missing DIMPLE_TEST_S3_ENDPOINT environment variable.");
        let bucket = env::var("DIMPLE_TEST_S3_BUCKET").expect("Missing DIMPLE_TEST_S3_BUCKET environment variable.");
        let prefix = env::var("DIMPLE_TEST_S3_PREFIX").expect("Missing DIMPLE_TEST_S3_PREFIX environment variable.");
        Self::new(&access_key, &secret_key, &region, &endpoint, &bucket, &prefix)
    }
}

impl Storage for S3Storage {
    fn put_object(&self, path: &str, contents: &[u8]) {
        // let path = format!("{}{}", self.prefix, path);
        let bucket = self.open_bucket();
        bucket.put_object(path, contents).unwrap();
    }

    fn get_object(&self, path: &str) -> Option<Vec<u8>> {
        // let path = format!("{}{}", self.prefix, path);
        let bucket  = self.open_bucket();
        let obj = bucket.get_object(&path).ok().map(|r| r.to_vec());
        obj
    }

    fn list_objects(&self, path: &str) -> Vec<String> {
        // let path = format!("{}{}", self.prefix, storage_prefix);
        let bucket = self.open_bucket();
        let results = bucket.list(path.to_string(), None)
            .unwrap()
            .iter()
            .flat_map(|r| r.contents.iter())
            // .map(|r| Self::strip_prefix(&r.key, &self.prefix))
            .map(|r| r.key.to_owned())
            .collect();
        results
    }
}

#[cfg(test)]
mod tests {
    use crate::sync::s3_storage::S3Storage;

    // After quite a bit of testing with the B2 API it seems that a leading /
    // is ignored / stripped for put, and is not compatible at all with list.
    // If a leading slash is included with list no results are returned, under
    // any circumstances, it seems.

    #[test]
    fn we_understand_leading_slash() {
        let s3 = S3Storage::default();
        // s3.put_object("honk/beep/myfile.txt", "aaa1fc41-6fad-4edd-81a0-3a0c82a1f8a1".as_bytes());
        // s3.put_object("/honk/beep/1-myfile.txt", "03e9de5d-efb9-4233-a8bf-ed011422fdb6".as_bytes());
        // s3.put_object("-dir-1.txt", "03e9de5d-efb9-4233-a8bf-ed011422fdb6".as_bytes());
        // s3.put_object("-dir-2.txt", "e359905f-dc3d-44a3-ab18-54d3e3c977d4".as_bytes());
        // dbg!(String::from_utf8(s3.get_object("/honk/beep/1-myfile.txt").unwrap()).unwrap());
        // dbg!(s3.list_objects(""));
        // dbg!(s3.list_objects("/"));
        // dbg!(s3.list_objects("dimple-dev-sync"));
        // dbg!(s3.list_objects("/001"));
    }

    // #[test]
    // fn prefix_doesnt_matter() {
    //     let mut s3 = S3Storage::default();
    //     s3.prefix = "".to_string();
    //     basics(&s3);
    //     s3.prefix = "/".to_string();
    //     basics(&s3);
    //     s3.prefix = "//".to_string();
    //     basics(&s3);
    //     s3.prefix = "1".to_string();
    //     basics(&s3);
    //     s3.prefix = "1/".to_string();
    //     basics(&s3);
    //     s3.prefix = "/1/".to_string();
    //     basics(&s3);
    // }

    // fn basics(storage: &S3Storage) {
    //     println!("Testing prefix {}", storage.prefix);
    //     storage.put_object("001/001.db", "faa44c67-92e2-411a-808b-cfd9fc9a263a".as_bytes());
    //     storage.put_object("001/002.db", "dcf51319-0a0d-4d1a-b825-26dea74d861b".as_bytes());
    //     storage.put_object("001/003.db", "1d922942-197d-4e72-9ccf-421e9f4d61aa".as_bytes());
    //     storage.put_object("002/002.db", "4a88dbef-349d-4845-bdab-6ed97457217d".as_bytes());
    //     storage.put_object("002/001.db", "fa63f446-d237-41bf-9739-30539223bf02".as_bytes());
    //     let objects = storage.list_objects("00");
    //     dbg!(&objects);
    //     assert!(objects.len() == 5);
    //     let objects = storage.list_objects("001/");
    //     assert!(objects.len() == 3);
    //     let objects = storage.list_objects("002/");
    //     assert!(objects.len() == 2);
    //     let content = storage.get_object("001/001.db");
    //     assert!(content == Some("faa44c67-92e2-411a-808b-cfd9fc9a263a".as_bytes().to_vec()));
    // }
}
