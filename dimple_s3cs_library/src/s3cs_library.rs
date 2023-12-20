// use std::{sync::Arc, path::{Path, PathBuf}, fs::{File}};

// use symphonia::{core::{codecs::CodecRegistry, probe::{Probe, Hint, ProbeResult}, io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions}, meta::{MetadataOptions, Tag, Value, MetadataRevision, Visual, ColorMode}, formats::{FormatOptions, Track, Cue}, units::TimeBase}, default};
// use walkdir::WalkDir;

use dimple_core::library::Library;
use image::DynamicImage;

pub struct S3csMusicLibrary {
    path: String,
}

impl S3csMusicLibrary {
    // let endpoint = "https://s3.us-west-004.backblazeb2.com".to_string();
    // let region_name = "us-west-004".to_string();
    // let access_key = "004b18e577e234a0000000002";
    // let secret_key = "K004EsSEVqEP+fQF6uQaiP40YsJ7PNs";
    // let bucket_name = "dimple-music";
    // let credentials = Credentials::new(Some(access_key), Some(secret_key), None, None, None).unwrap();
    // let region = Region::Custom { 
    //     region: region_name, 
    //     endpoint 
    // };
    // let bucket = Bucket::new(
    //     bucket_name,
    //     region,
    //     credentials,
    // ).unwrap();

    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl Library for S3csMusicLibrary {
    fn name(&self) -> String {
        todo!()
    }

    fn releases(&self) -> std::sync::mpsc::Receiver<dimple_core::model::Release> {
        todo!()
    }

    fn image(&self, _image: &dimple_core::model::Image) -> Result<DynamicImage, String> {
        todo!()
    }

    fn stream(&self, _track: &dimple_core::model::Track) -> Result<Vec<u8>, String> {
        todo!()
    }
}


