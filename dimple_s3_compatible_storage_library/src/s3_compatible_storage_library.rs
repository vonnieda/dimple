use std::path::Path;

use dimple_core::library::Library;
use dimple_core::model::Release;
use image::DynamicImage;
use s3::serde_types::Object;
use serde::Deserialize;
use serde::Serialize;
use s3::{Bucket, creds::Credentials, Region};
use symphonia::core::codecs::CODEC_TYPE_NULL;
use symphonia::core::codecs::CodecRegistry;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSource;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::default::get_codecs;

pub struct S3CompatibleStorageLibrary {
    config: S3CompatibleStorageLibraryConfig,
}

#[derive(Deserialize, Debug, Serialize, Default, Clone)]
pub struct S3CompatibleStorageLibraryConfig {
    pub ulid: String,
    pub name: String,
    pub endpoint: String,
    pub region_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub bucket_name: String,
    pub prefix: String,
}

impl S3CompatibleStorageLibrary {
    pub fn new(config: &S3CompatibleStorageLibraryConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    fn bucket(&self) -> Bucket {
        let credentials = Credentials::new(Some(&self.config.access_key), 
            Some(&self.config.secret_key), None, None, None).unwrap();
        let region = Region::Custom { 
            region: self.config.region_name.to_string(), 
            endpoint: self.config.endpoint.to_string(),
        };
        Bucket::new(
            &self.config.bucket_name,
            region,
            credentials,
        ).unwrap()
    }
}

impl Library for S3CompatibleStorageLibrary {
    fn name(&self) -> String {
        self.config.name.clone()
    }

    fn releases(&self) -> std::sync::mpsc::Receiver<dimple_core::model::Release> {
        let codec_registry = CodecRegistry::default();


        let bucket = self.bucket();
        let results = bucket.list_page(self.config.prefix.clone(), 
            None,
            None,
            None,
            Some(100)).unwrap();
        for obj in results.0.contents {
            let key = obj.key;
            let mut hint = Hint::new();
            if let Some(extension) = key.split('.').last() {
                hint.with_extension(extension);
            }
            let media_source = ObjectMediaSource {
                bucket: bucket.clone(),
                key: key.clone(),
            };
            let media_source_stream = MediaSourceStream::new(Box::new(media_source), 
                Default::default());
            
            // Use the default options for metadata and format readers.
            let meta_opts: MetadataOptions = Default::default();
            let fmt_opts: FormatOptions = Default::default();

            // Probe the media source.
            let probed = symphonia::default::get_probe()
                .format(&hint, media_source_stream, &fmt_opts, &meta_opts)
                .expect("unsupported format");

            // Get the instantiated format reader.
            let mut format = probed.format;

            let mut metadata = format.metadata();

            if let Some(metadata) = metadata.skip_to_latest() {
                log::info!("{} {:?}", key, metadata.tags());
            }

            // Find the first audio track with a known (decodeable) codec.
            let track = format
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                .expect("no supported audio tracks");

            // Use the default options for the decoder.
            let dec_opts: DecoderOptions = Default::default();

            // Create a decoder for the track.
            let mut decoder = symphonia::default::get_codecs()
                .make(&track.codec_params, &dec_opts)
                .expect("unsupported codec");

            // Store the track identifier, it will be used to filter packets.
            let track_id = track.id;
        }

        let (_sender, receiver) = std::sync::mpsc::channel::<Release>();
        receiver
    }

    fn image(&self, _image: &dimple_core::model::Image) -> Result<DynamicImage, String> {
        todo!()
    }

    fn stream(&self, _track: &dimple_core::model::Track) -> Result<Vec<u8>, String> {
        todo!()
    }
}

struct ObjectMediaSource {
    bucket: Bucket,
    key: String,
}

impl MediaSource for ObjectMediaSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        match self.bucket.head_object(&self.key) {
            Ok((result, _)) => {
                Some(result.content_length.unwrap().unsigned_abs())
            },
            Err(_) => todo!(),
        }
    }
}

impl std::io::Read for ObjectMediaSource {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!()
    }
}

impl std::io::Seek for ObjectMediaSource {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!()
    }
}
