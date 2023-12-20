/**
 * keyID: 004b18e577e234a0000000002
 * keyName: dimple
 * applicationKey: K004EsSEVqEP+fQF6uQaiP40YsJ7PNs
 */
// cargo run --example sync --no-default-features --features sync-native-tls

use s3::error::S3Error;
use s3::{Bucket, Region};
use s3::creds::Credentials;

fn main() {
    let access_key = "004b18e577e234a0000000002";
    let secret_key = "K004EsSEVqEP+fQF6uQaiP40YsJ7PNs";
    let credentials = Credentials::new(Some(access_key), Some(secret_key), None, None, None).unwrap();
    // Custom region requires valid region name and endpoint
    let region_name = "us-west-004".to_string();
    let endpoint = "https://s3.us-west-004.backblazeb2.com".to_string();
    let region = Region::Custom { 
        region: region_name, 
        endpoint 
    };
    let bucket = Bucket::new(
        "dimple-music",
        region,
        credentials,
    ).unwrap();

    let results = bucket.list("".to_string(), None).unwrap();

    let mut total = 0;
    let mut total_size = 0;
    for result in results {
        // dbg!(result.contents.len());
        for thinger in result.contents {
            dbg!(thinger);
            // total += 1;
            // total_size += thinger.size;
        }
    }
    // dbg!(total);
    // dbg!(total_size);

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
}

