use std::fs::File;

use symphonia::core::{errors::Error, formats::FormatOptions, io::MediaSourceStream, meta::{MetadataOptions, StandardTagKey, Tag, Visual}, probe::Hint};

#[derive(Clone, Debug)]
pub struct ScannedFile {
    pub path: String,
    pub tags: Vec<Tag>,
    pub visuals: Vec<Visual>,
    pub length_ms: Option<u64>,
}

impl ScannedFile {
    pub fn new(path: &str) -> Result<ScannedFile, Error> {
        let path = std::fs::canonicalize(path).unwrap();

        let media_source = File::open(&path).unwrap();
        let media_source_stream =
            MediaSourceStream::new(Box::new(media_source), Default::default());

        // Use the default options for metadata and format readers.
        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let mut hint = Hint::new();
        if let Some(extension) = path.extension() {
            hint.with_extension(extension.to_str().unwrap());
        }
        
        // Probe the media source.
        let probed = symphonia::default::get_probe()
            .format(&hint, media_source_stream, &fmt_opts, &meta_opts);

        if let Err(e) = probed {
            log::error!("{:?} {:?}", &path, e.to_string());
            return Err(e)
        }

        let mut probed = probed.unwrap();

        // Get the instantiated format reader.
        let mut format = probed.format;

        // Collect all of the tags from both the file and format metadata
        let mut tags: Vec<Tag> = vec![];
        let mut visuals: Vec<Visual> = vec![];

        if let Some(metadata) = probed.metadata.get() {
            if let Some(metadata) = metadata.current() {
                tags.extend(metadata.tags().to_owned());
                visuals.extend(metadata.visuals().to_owned());
            }
        }

        let metadata = format.metadata();

        if let Some(metadata) = metadata.current() {
            tags.extend(metadata.tags().to_owned());
            visuals.extend(metadata.visuals().to_owned());
        }

        let mut length_ms = None;
        if let Some(track) = format.tracks().get(0) {
            if let Some(time_base) = track.codec_params.time_base {
                if let Some(n_frames) = track.codec_params.n_frames {
                    let length = time_base.calc_time(n_frames);
                    length_ms = Some((length.seconds * 1000) + ((length.frac * 1000.) as u64));
                }
            }
        }

        let media_file = ScannedFile {
            path: path.to_str().unwrap().to_string(),
            tags,
            visuals,
            length_ms,
        };

        Ok(media_file)
    }

    pub fn tag(&self, key: StandardTagKey) -> Option<String> {
        self.tags.iter().find_map(|t| {
            if let Some(std_key) = t.std_key {
                if std_key == key {
                    return Some(t.value.to_string())
                }
            }
            None
        })
    }
}

