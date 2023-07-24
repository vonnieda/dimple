use std::time::Duration;

use cpal::{Data, Sample, SampleFormat, FromSample};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// https://github.com/RustAudio/rodio/issues/392
// https://github.com/tesselode/kira
// https://github.com/woelper/lynx
// https://github.com/RustAudio/cpal/issues/755
// rubato for resampling
// I think using https://github.com/tuzz/audio_mixer as an example to learn
// how to use CPAL is gonna have to be it.
// And probably symphonia.
// But using https://github.com/tesselode/kira might get us going...
// Kira examples worked, and I already know how to deal with positioning.
// Might be worth going that route for now.

fn main() {
    let mut builder = env_logger::Builder::new();
    builder.filter_level(log::LevelFilter::Info);
    builder.format_timestamp_millis();
    builder.init();


    let host = cpal::default_host();
    let device = host.default_output_device().expect("no output device");

    let mut supported_configs_range = device.supported_output_configs()
        .expect("error while querying configs");
    let supported_config = supported_configs_range.next()
        .expect("no supported config?!")
        .with_max_sample_rate();
    let err_fn = |err| eprintln!("an error occurred on the output audio stream: {}", err);
    let sample_format = supported_config.sample_format();
    let config = supported_config.into();
    let source = SineWave::new(440.0);
    let stream = match sample_format {
        SampleFormat::F32 => device.build_output_stream(&config, source.write, err_fn, None),
        // SampleFormat::I16 => device.build_output_stream(&config, write_silence::<i16>, err_fn, None),
        // SampleFormat::U16 => device.build_output_stream(&config, write_silence::<u16>, err_fn, None),
        sample_format => panic!("Unsupported sample format '{sample_format}'")
    }.unwrap();
    stream.play().unwrap();    
    std::thread::sleep(Duration::from_secs(2));
}

struct SineWave {

}

impl SineWave {
    pub fn new(freq: f32) -> Self {
        Self {

        }
    }

    pub fn write(&self, data: &mut [f32], info: &cpal::OutputCallbackInfo) {
        // println!("{} {:#?}", data.len(), info);
        // for sample in data.iter_mut() {
        //     let inst = info.timestamp().callback;
        //     inst.add(duration)
            
        //     // let nanos = inst.
        //     // *sample = info.timestamp().callback.
        // }
    }    
}

