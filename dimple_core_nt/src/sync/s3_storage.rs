use s3::{creds::Credentials, Bucket, Region};
use std::fs::{self, File};

pub struct S3Storage {
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
    pub endpoint: String,
    pub bucket: String,
    pub prefix: String,
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

    pub fn put_file(&self, s3_key: &str, file_path: &str) {
        let bucket = self.open_bucket();
        let content = fs::read(file_path).unwrap();
        let response_code = bucket.put_object(s3_key, &content).unwrap().status_code();
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

}
