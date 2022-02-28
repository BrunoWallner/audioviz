use core::f32::consts::PI;

use crate::fft;

//
// Rewritten from: "https://github.com/phip1611/lowpass-filter"
// and added highpass_filter with the help of
// "https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter"
// and
// "https://en.wikipedia.org/wiki/High-pass_filter#Simple_infinite_impulse_response_filter"
//

/*
// https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter
pub fn lowpass_filter(data: &[f32], sampling_rate: f32, cutoff_frequency: f32) -> Vec<f32> {
    let rc = 1.0 / (cutoff_frequency * 2.0 * PI);
    // time per sample
    let dt = 1.0 / sampling_rate;

    let mut y: Vec<f32> = vec![0.0; data.len()];
    let alpha = dt / (rc + dt);

    y[0] = alpha * data[0];
    for i in 1..data.len() {
        y[i] = y[i - 1] + alpha * (data[i] - y[i - 1]);
    }
    
    y
}

// https://en.wikipedia.org/wiki/High-pass_filter#Simple_infinite_impulse_response_filter
pub fn highpass_filter(data: &[f32], sampling_rate: f32, cutoff_frequency: f32) -> Vec<f32> {
    let rc = 1.0 / (cutoff_frequency * 2.0 * PI);
    // time per sample
    let dt = 1.0 / sampling_rate;

    let mut y: Vec<f32> = vec![0.0; data.len()];
    let alpha = rc / (rc + dt);

    y[1] = data[1];

    for i in 1..data.len() {
        y[i] = alpha * (y[i-1] + data[i] - data[i-1])
    }

    y
}
*/

pub fn lowpass_filter(data: &[f32], sampling_rate: f32, cutoff_start_freq: f32, cutoff_end_freq: f32) -> Vec<f32> {
    assert!(cutoff_end_freq >= cutoff_start_freq);

    assert!(cutoff_start_freq <= sampling_rate / 2.0 && cutoff_end_freq <= sampling_rate / 2.0);

    let len = data.len();
    let spectrum_len = len / 2;

    let mut spectrum = fft::forward(&data);
    assert!(len == spectrum.len());

    let start: usize = (spectrum_len as f32 * (cutoff_start_freq / sampling_rate * 2.0)) as usize;
    let end: usize = (spectrum_len as f32 * (cutoff_end_freq / sampling_rate * 2.0)) as usize;
    let diff = end - start;

    // what to add to position in each iteration
    let step: f32 = PI / diff as f32;

    let mut position: f32 = 0.0;
    for i in start..=end {
        let mul = (position.cos() + 1.0) / 2.0;
        spectrum[i] *= mul;
        spectrum[len - i - 1] *= mul;

        position += step;
    }
    for i in end..spectrum_len {
        spectrum[i] *= 0.0;
        spectrum[len - i - 1] *= 0.0;
    }

    let data = fft::inverse(&spectrum);

    fft::get_real(&data)
}

pub fn highpass_filter(data: &[f32], sampling_rate: f32, cutoff_start_freq: f32, cutoff_end_freq: f32) -> Vec<f32> {
    assert!(cutoff_end_freq <= cutoff_start_freq);

    assert!(cutoff_start_freq <= sampling_rate / 2.0 && cutoff_end_freq <= sampling_rate / 2.0);

    let len = data.len();
    let spectrum_len = len / 2;

    let mut spectrum = fft::forward(&data);
    assert!(len == spectrum.len());

    let start: usize = (spectrum_len as f32 * (cutoff_end_freq / sampling_rate * 2.0)) as usize;
    let end: usize = (spectrum_len as f32 * (cutoff_start_freq / sampling_rate * 2.0)) as usize;
    let diff = end - start;

    // what to subract from position in each iteration
    let step: f32 = PI / diff as f32;

    let mut position: f32 = PI;
    for i in start..=end {
        let mul = (position.cos() + 1.0) / 2.0;
        spectrum[i] *= mul;
        spectrum[len - i - 1] *= mul;

        position -= step;
    }
    for i in 0..=start {
        spectrum[i] *= 0.0;
        spectrum[len - i - 1] *= 0.0;
    }

    let data = fft::inverse(&spectrum);

    fft::get_real(&data)
}

pub fn bandpass_filter(
    data: &[f32], 
    sampling_rate: f32,
    low_cutoff_start_freq: f32,
    low_cutoff_end_freq: f32,
    high_cutoff_start_freq: f32,
    high_cutoff_end_freq: f32,
) -> Vec<f32> {
    // NAN
    assert!(low_cutoff_end_freq >= low_cutoff_start_freq);
    assert!(high_cutoff_end_freq >= high_cutoff_start_freq);

    assert!(low_cutoff_start_freq <= sampling_rate / 2.0 && low_cutoff_end_freq <= sampling_rate / 2.0);
    assert!(high_cutoff_start_freq <= sampling_rate / 2.0 && high_cutoff_end_freq <= sampling_rate / 2.0);

    let len = data.len();
    let spectrum_len = len / 2;

    let mut spectrum = fft::forward(&data);
    assert!(len == spectrum.len());

    let low_start: usize = (spectrum_len as f32 * (low_cutoff_start_freq / sampling_rate * 2.0)) as usize;
    let low_end: usize = (spectrum_len as f32 * (low_cutoff_end_freq / sampling_rate * 2.0)) as usize;
    let low_diff = low_end - low_start;

    let high_start: usize = (spectrum_len as f32 * (high_cutoff_start_freq / sampling_rate * 2.0)) as usize;
    let high_end: usize = (spectrum_len as f32 * (high_cutoff_end_freq / sampling_rate * 2.0)) as usize;
    let high_diff = high_end - high_start;

    // what to subtract from low_position in each iteration
    let low_step: f32 = PI / low_diff as f32;

    // what to add to high_position in each iteration
    let high_step: f32 = PI / high_diff as f32;

    // smooth transition between cut and not cut freqs
    // lowcut
    let mut low_position: f32 = PI;
    for i in low_start..=low_end {
        let mul = (low_position.cos() + 1.0) / 2.0;
        spectrum[i] *= mul;
        spectrum[len - i - 1] *= mul;

        low_position -= low_step;
    }
    // highcut
    let mut high_position: f32 = 0.0;
    for i in high_start..=high_end {
        let mul = (high_position.cos() + 1.0) / 2.0;
        spectrum[i] *= mul;
        spectrum[len - i - 1] *= mul;

        high_position += high_step;
    }

    // mutes freqs that are beyond threshold
    // left from lowcut
    for i in 0..=low_start {
        spectrum[i] *= 0.0;
        spectrum[len - i - 1] *= 0.0;
    }
    // right from highcut
    for i in high_end..=spectrum_len {
        spectrum[i] *= 0.0;
        spectrum[len - i - 1] *= 0.0;
    }

    let data = fft::inverse(&spectrum);

    fft::get_real(&data)
}
