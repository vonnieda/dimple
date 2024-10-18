use std::fs::{self, File};

/// Sync a Library with an S3 compatible storage target. Allows multiple
/// devices to share the same library.

use s3::{Bucket, Region};
use s3::creds::Credentials;

use crate::library::Library;

pub struct Sync {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String,
}

impl Sync {
    pub fn new(access_key: &str, secret_key: &str, region: &str, endpoint: &str,
        bucket: &str, prefix: &str) -> Sync {

        Sync {
            access_key: access_key.to_owned(),
            secret_key: secret_key.to_owned(),
            region: region.to_owned(),
            endpoint: endpoint.to_owned(),
            bucket: bucket.to_owned(),
            prefix: prefix.to_owned(),
        }
    }
    
    /// Sync library to the specified S3 compatible storage target. 
    pub fn sync(&self, library: &Library) {
        let library_uuid = library.uuid();
        library.backup(&library_uuid);
        self.put_file(&library_uuid, &library_uuid);
    }

    fn put_file(&self, key: &str, file: &str) {
        let bucket = self.open_bucket();
        let content = fs::read(file).unwrap();
        let response_code = bucket.put_object(key, &content).unwrap().status_code();
        assert_eq!(response_code, 200);
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

    fn list_remote_dbs(&self) -> Vec<String> {
        let bucket = self.open_bucket();

        let prefix = format!("{}/dimple.db/", self.prefix);
        let results = bucket.list(prefix.to_string(), None).unwrap();

        for result in results {
            for obj in result.contents {
                println!("{:?} {} {} {}", obj.e_tag, obj.key, obj.last_modified, obj.size);
            }
        }

        vec![]
    }
}


// let s3_path = "test.file";
// let test = b"I'm going to S3!";

// let response_data = bucket.put_object(s3_path, test)?;
// assert_eq!(response_data.status_code(), 200);

// let response_data = bucket.get_object(s3_path)?;
// assert_eq!(response_data.status_code(), 200);
// assert_eq!(test, response_data.as_slice());

// let response_data = bucket.get_object_range(s3_path, 100, Some(1000))?;
// assert_eq!(response_data.status_code(), 206);
// let (head_object_result, code) = bucket.head_object(s3_path)?;
// assert_eq!(code, 200);
// assert_eq!(
//     head_object_result.content_type.unwrap_or_default(),
//     "application/octet-stream".to_owned()
// );

// let response_data = bucket.delete_object(s3_path)?;
// assert_eq!(response_data.status_code(), 204);
