//! Fast Fourier Transform algorithm
//! necessary to transform audio-data to a representation in the frequency domain
//! 
//! dependency of `spectrum`
//! 
use rustfft::FftPlanner;
pub use rustfft::num_complex::Complex;

pub fn forward(data: &[f32]) -> Vec<Complex<f32>> {
    let length = data.len();

    // conversion to complex numbers
    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for d in data {
        buffer.push(Complex{re: *d, im: 0.0});
    }

    // creates a planner
    let mut planner = FftPlanner::<f32>::new();

    // creates a FFT
    let fft = planner.plan_fft_forward(length);

    //input.append(&mut data.to_vec());

    fft.process(&mut buffer);

    buffer
}

pub fn inverse(data: &[Complex<f32>]) -> Vec<Complex<f32>> {
    let length = data.len();

    let mut data: Vec<Complex<f32>> = data.to_vec();


    // creates a planner
    let mut planner = FftPlanner::<f32>::new();

    // creates a FFT
    let fft = planner.plan_fft_inverse(length);


    fft.process(&mut data);

    data.to_vec()
}

pub fn remove_mirroring(data: &[f32]) -> Vec<f32> {
    let len = data.len() / 2 + 1;
    data[..len].to_vec()
}

/// normalizes complex array to real one
pub fn normalize(data: &[Complex<f32>]) -> Vec<f32> {
    let norm = data
        .iter()
        .map(|x| x.norm())
        .collect();

    norm
}

// only extract real numbers out of complex ones
pub fn get_real(data: &[Complex<f32>]) -> Vec<f32> {
    let len: f32 = data.len() as f32;
    let norm = data
        .iter()
        .map(|x| x.re / len)
        .collect();

    norm
}