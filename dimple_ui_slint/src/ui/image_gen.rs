use std::f32::consts::PI;

use dimple_core::player::Song;
use image::{DynamicImage, ImageBuffer};
use sonogram::{ColourGradient, FrequencyScale, RGBAColour, SpecOptionsBuilder};
use tiny_skia::*;

pub fn gen_fuzzy_circles(width: u32, height: u32) -> DynamicImage {
    let output_width = width;
    let output_height = height;
    let width = 128;
    let height = 128;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    let mut paint = tiny_skia::Paint::default();
    for i in 0..50 {
        paint.set_color_rgba8(
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
        );
        let circle = tiny_skia::PathBuilder::from_circle(
            fakeit::misc::random(0., width as f32), 
            fakeit::misc::random(0., height as f32), 
            fakeit::misc::random(2., width as f32 / 3.), 
        ).unwrap();
        pixmap.fill_path(&circle, &paint, tiny_skia::FillRule::Winding, Default::default(), None);
    }

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(width, height, pixmap.data().to_vec()).unwrap();
    let image = image::imageops::blur(&image, 9.0);

    let dyn_image = DynamicImage::ImageRgba8(image);
    dyn_image.resize(output_width, output_height, image::imageops::FilterType::Nearest)
}

pub fn gen_fuzzy_rects(width: u32, height: u32) -> DynamicImage {
    let output_width = width;
    let output_height = height;
    let width = 128;
    let height = 128;
    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    let mut paint = tiny_skia::Paint::default();
    for i in 0..50 {
        paint.set_color_rgba8(
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
            fakeit::misc::random(0, 255),
        );
        let rect = tiny_skia::PathBuilder::from_rect(Rect::from_xywh(
            fakeit::misc::random(0., width as f32), 
            fakeit::misc::random(0., height as f32), 
            fakeit::misc::random(2., width as f32 / 3.), 
            fakeit::misc::random(2., height as f32 / 3.), 
        ).unwrap());
        pixmap.fill_path(&rect, &paint, tiny_skia::FillRule::Winding, Default::default(), None);
    }

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(width, height, pixmap.data().to_vec()).unwrap();
    let image = image::imageops::blur(&image, 7.0);
    
    let dyn_image = DynamicImage::ImageRgba8(image);
    dyn_image.resize(output_width, output_height, image::imageops::FilterType::Nearest)
}

/// Applies a Hann window (<https://en.wikipedia.org/wiki/Window_function#Hann_and_Hamming_windows>)
/// to an array of samples.
///
/// ## Return value
/// New vector with Hann window applied to the values.
pub fn hann_window(samples: &[f32]) -> Vec<f32> {
    let mut windowed_samples = Vec::with_capacity(samples.len());
    let samples_len_f32 = samples.len() as f32;
    for (i, sample) in samples.iter().enumerate() {
        let two_pi_i = 2.0 * PI * i as f32;
        let idontknowthename = (two_pi_i / samples_len_f32).cos();
        let multiplier = 0.5 * (1.0 - idontknowthename);
        windowed_samples.push(multiplier * sample)
    }
    windowed_samples
}

// https://github.com/freestrings/waveform/blob/master/src/main.rs#L148
///https://en.wikipedia.org/wiki/Root_mean_square
///http://m.blog.naver.com/pkw00/220226903866
fn rms(samples: &[f32]) -> f32 {
    let sum = samples.iter().fold(0.0, |acc, sample| acc + (*sample as f64).powi(2));
    let len = samples.len() as f64;
    (sum / len).sqrt() as f32
}

// https://github.com/freestrings/waveform
pub fn gen_song_waveform(song: &Song, width: u32, height: u32) -> DynamicImage {
    assert!(song.channel_count == 1 || song.channel_count == 2);

    // Left is channel 0
    let l_samples = song.samples.get(0).unwrap();
    // Right is channel 1, or if mono, duplicate the left channel
    let r_samples = song.samples.get(1).or(song.samples.get(0)).unwrap();
    assert!(l_samples.len() == r_samples.len());

    let l_max = rms(&l_samples);
    let r_max = rms(&r_samples);

    let rect_width = 1;
    let num_rects: usize = (width / rect_width) as usize;
    let samples_per_rect: usize = l_samples.len() / num_rects;

    let mut pixmap = tiny_skia::Pixmap::new(width, height).unwrap();
    let mut paint = Paint::default();
    paint.set_color_rgba8(0x8a, 0x65, 0x8a, 200);
    for i in 0..num_rects {
        let window_start = i * samples_per_rect;
        let window_end = i * samples_per_rect + samples_per_rect;

        let l_windowed = hann_window(&l_samples[window_start..window_end]);
        let l_rms = rms(&l_windowed);
        let x = i as f32 * rect_width as f32;
        let y = height as f32 / 2.;
        let w = rect_width as f32;
        let h = (height as f32 / 2.) * (l_rms / l_max);
        let rect = PathBuilder::from_rect(Rect::from_xywh(x, y, w, h).unwrap());
        pixmap.fill_path(&rect, &paint, tiny_skia::FillRule::Winding, Default::default(), None);

        let r_windowed = hann_window(&r_samples[window_start..window_end]);
        let r_rms = rms(&r_windowed);
        let x = i as f32 * rect_width as f32;
        let y = height as f32 / 2.;
        let w = rect_width as f32;
        let h = (height as f32 / 2.)  * (r_rms / r_max);
        let rect = PathBuilder::from_rect(Rect::from_xywh(x, y - h, w, h).unwrap());
        pixmap.fill_path(&rect, &paint, tiny_skia::FillRule::Winding, Default::default(), None);
    }

    let image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::from_raw(width, height, pixmap.data().to_vec()).unwrap();    
    DynamicImage::ImageRgba8(image)
}

pub fn gen_song_spectrogram(song: &Song, width: u32, height: u32) -> DynamicImage {
    assert!(song.channel_count == 1 || song.channel_count == 2);

    // Left is channel 0
    let l_samples = song.samples.get(0).unwrap();
    // Right is channel 1, or if mono, duplicate the left channel
    let r_samples = song.samples.get(1).or(song.samples.get(0)).unwrap();
    assert!(l_samples.len() == r_samples.len());

    let spec_builder = SpecOptionsBuilder::new(2048)
        .load_data_from_memory_f32(l_samples.clone(), song.sample_rate);
    let mut spectrograph = spec_builder.build().unwrap().compute();
    // let mut gradient = ColourGradient::new();
    // gradient.add_colour(RGBAColour::new(216, 47, 216, 255)); // White
    // gradient.add_colour(RGBAColour::new(0x8a, 0x65, 0x8a, 200)); // White
    // gradient.add_colour(RGBAColour::new(216, 47, 216, 0)); // Black
    // let mut gradient = ColourGradient::rainbow_theme();
    // let mut gradient = ColourGradient::black_white_theme();
    // let mut gradient = ColourGradient::default_theme();
    let mut gradient = ColourGradient::audacity_theme();
    let rgba = spectrograph.to_rgba_in_memory(FrequencyScale::Log, 
        &mut gradient, 
        width as usize, 
        height as usize);

    let image: ImageBuffer<image::Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(width, height, rgba).unwrap();    
    DynamicImage::ImageRgba8(image)
}

