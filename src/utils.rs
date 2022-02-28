//! general utilities that help to process audio data

/// seperates 1 dimensional interleaved audio stream to 2D vector of audiodata of each channel
pub fn seperate_channels(data: &[f32], channel_count: usize) -> Vec<Vec<f32>> {
    let mut buffer: Vec<Vec<f32>> = vec![vec![]; channel_count];

    for chunked_data in data.chunks(channel_count) {
        for (i, v) in chunked_data.iter().enumerate() {
            buffer[i].push(*v);
        }
    }

    buffer
}

#[cfg(feature = "apodize")]
pub fn apodize(data: &mut Vec<f32>) {
    let window = apodize::hanning_iter(data.len()).collect::<Vec<f64>>();
    for (i, value) in data.iter_mut().enumerate() {
        *value *= window[i] as f32;
    }
}

#[cfg(all(feature = "apodize", feature = "fft"))]
use crate::fft::Complex;

#[cfg(all(feature = "apodize", feature = "fft"))]
pub fn apodize_complex(data: &mut Vec<Complex<f32>>) {
    let window = apodize::hanning_iter(data.len()).collect::<Vec<f64>>();
    for (i, value) in data.iter_mut().enumerate() {
        value.re *= window[i] as f32;
    }
}