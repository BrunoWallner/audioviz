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

pub fn combine_channels(mut data: Vec<Vec<f32>>) -> Vec<f32> {
    let mut buffer: Vec<f32> = Vec::new();
    loop {
        let mut end: Vec<bool> = vec![false; data.len()];
        for (i, d) in data.iter_mut().enumerate() {
            if d.len() > 0 {
                buffer.push(d.remove(0));
            } else {
                end[i] = true
            }
        }
        if !end.contains(&false) {
            break;
        }
    }
    buffer
}

#[cfg(feature = "apodize")]
pub fn apodize(data: &[f32]) -> Vec<f32> {
    let window = apodize::hamming_iter(data.len()).collect::<Vec<f64>>();
    let mut buffer = Vec::with_capacity(data.len());
    for (i, value) in data.iter().enumerate() {
        buffer.push(value * window[i] as f32);
    }
    buffer
}

#[cfg(feature = "apodize")]
pub fn inverse_apodize(data: &[f32]) -> Vec<f32> {
    let window = apodize::hamming_iter(data.len()).collect::<Vec<f64>>();
    let mut buffer = Vec::with_capacity(data.len());
    for (i, value) in data.iter().enumerate() {
        // let divisor = max_float(0.001, window[i] as f32);
        let divisor = window[i] as f32;
        // let value = if divisor == 0.0 { 0.0 } else { value / divisor };
        let value = value / divisor;

        buffer.push(value);
    }
    buffer
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
