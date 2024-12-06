use std::env;

use s3::{creds::Credentials, Bucket, Region};

use super::storage::Storage;

pub struct S3Storage {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String, // TODO currently ignored, may add support back later
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
