use rustfft::{num_complex::Complex, FftPlanner};

pub fn process(data: &[f32]) -> Vec<f32> {
    let mut planner = FftPlanner::new();

    let mut buffer: Vec<f32> = Vec::with_capacity(data.len() / 2);
    let mut complex_buffer: Vec<Complex<f32>> = data
        .into_iter()
        .map(|x| Complex { re: *x, im: 0.0 })
        .collect();


    let fft = planner.plan_fft_forward(complex_buffer.len());

    fft.process(&mut complex_buffer[..]);
    complex_buffer
        .iter()
        .for_each(|x| buffer.push(x.norm()));

    // remove mirroring
    buffer =
        buffer[0..(buffer.len() as f32 * 0.5) as usize].to_vec();

    buffer
}