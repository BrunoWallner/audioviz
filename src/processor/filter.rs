use core::f32::consts::PI;

//
// Rewritten from: "https://github.com/phip1611/lowpass-filter"
// and added highpass_filter with the help of
// "https://en.wikipedia.org/wiki/Low-pass_filter#Simple_infinite_impulse_response_filter"
// and
// "https://en.wikipedia.org/wiki/High-pass_filter#Simple_infinite_impulse_response_filter"
//

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

    for i in 2..data.len() {
        y[i] = alpha * (y[i-1] + data[i] - data[i-1]);
    }

    y
}