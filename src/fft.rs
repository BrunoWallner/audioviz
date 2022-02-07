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

#[cfg(test)]
mod tests {
    #[test]
    fn fft() {
        let mut buffer: Vec<f32> = Vec::new();

        let mut x: f32 = 0.0;
        for _ in 0..16 {
            buffer.push(x.sin());
            x += 0.1;
        }
    
        let fft = super::process(&buffer);

        assert_eq!(
            fft,
            vec![
                9.78363,
                2.9537609,
                1.4024371,
                0.95359206,
                0.74589825,
                0.63307375,
                0.569189,
                0.5359103,         
            ]
        )
    }
}