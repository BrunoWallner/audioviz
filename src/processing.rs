use rustfft::{num_complex::Complex, FftPlanner};
use crate::config::Config;
use splines::{Interpolation, Key, Spline};

/// puts buffer into FFT alogrithm and applies filters and modifiers to it
pub fn convert_buffer(
    input_buffer: &[f32],
    config: Config,
) -> Vec<f32> {
    let input_buffer = apodize(input_buffer);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(input_buffer.len());

    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for i in input_buffer.iter() {
        buffer.push(Complex {
            re: *i,
            im: *i,
        });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    for i in buffer.iter() {
        output_buffer.push(i.norm())
    }

    // remove mirroring
    let output_buffer = output_buffer[0..(output_buffer.len() as f32 * 0.25) as usize].to_vec();

    // max frequency
    let percentage: f32 = config.max_frequency as f32 / 20_000_f32;
    let output_buffer = output_buffer[0..(output_buffer.len() as f32 * percentage) as usize].to_vec();

    let output_buffer = normalize(output_buffer, config.volume);
    let mut output_buffer = volume_distribution(&output_buffer, &vec![1.0, 1.0, 1.0, 1.0, 1.0, 1.0, ]);

    smooth(&mut output_buffer, config.smoothing_amount, config.smoothing_size);

    let output_buffer = apply_resolution(&output_buffer, config.resolution);

    output_buffer
}


fn apodize(buffer: &[f32]) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
}

#[allow(clippy::needless_range_loop)]
fn normalize(buffer: Vec<f32>, volume: f32) -> Vec<f32> {
    let mut output_buffer: Vec<f32> = vec![ 0.0; buffer.len() ];

    let mut pos_index: Vec<(usize, f32)> = Vec::new();

    for i in 0..buffer.len() {
        let offset: f32 = (buffer.len() as f32 / (i + 1) as f32).sqrt();
        if ((i as f32 * offset) as usize) < buffer.len() {
            // space normalisation and space distribution
            let pos = (i as f32 * offset) as usize;

            // volume normalisation
            let volume_offset: f32 = (output_buffer.len() as f32 / (pos + 1) as f32).sqrt();
            let y = buffer[i] / volume_offset.powi(3) * 0.01;

            pos_index.push( ((pos as f32) as usize, y) );
        }
    }

    // Interpolation
    let mut points: Vec<Key<f32, f32>> = Vec::new();
    for val in pos_index.iter() {
        let x = val.0 as f32;
        let y = val.1 * volume;
        points.push(Key::new(x, y, Interpolation::Linear));
    }

    let spline = Spline::from_vec(points);

    for i in 0..output_buffer.len() {
        let v = spline.clamped_sample(i as f32).unwrap_or(0.0);
        output_buffer[i] = v;
    }

    output_buffer
}

fn apply_resolution(buffer: &Vec<f32>, resolution: f32) -> Vec<f32> {
    if resolution > 1.0 {
        let mut output_buffer: Vec<f32> = vec![0.0; (buffer.len() as f32 * resolution ) as usize];

        let mut points: Vec<Key<f32, f32>> = Vec::new();
        for (i, val) in buffer.iter().enumerate() {
            points.push(Key::new(i  as f32 * resolution, *val, Interpolation::Linear));
        }
    
        let spline = Spline::from_vec(points);
    
        for i in 0..output_buffer.len() {
            let v = spline.clamped_sample(i as f32).unwrap_or(0.0);
            output_buffer[i] = v;
        }
    
        return output_buffer;
    }
    else if resolution < 1.0 {
        let mut output_buffer: Vec<f32> = vec![0.0; (buffer.len() as f32 * resolution ) as usize];
        let offset = output_buffer.len() as f32 / buffer.len() as f32;
        for (i, val) in buffer.iter().enumerate() {
            let pos = (i as f32 * offset) as usize;
            if pos < output_buffer.len() {
                if output_buffer[pos] < *val {
                    output_buffer[pos] = *val;
                }
            }
        }

        return output_buffer;
    }
    else {
        return buffer.to_vec();
    }

}

fn volume_distribution(buffer: &Vec<f32>, distribution: &Vec<f32>) -> Vec<f32> {
    let mut output_buffer: Vec<f32> = vec![0.0; buffer.len()]; // must share same len with buffer
    
    let mut dis_points: Vec<Key<f32, f32>> = Vec::new();
    let step = output_buffer.len() as f32 / (distribution.len() - 1) as f32;

    for (i, val) in distribution.iter().enumerate() {
        dis_points.push(Key::new(i as f32 * step, *val, Interpolation::Linear));
    }
    let dis_spline = Spline::from_vec(dis_points);

    for i in 0..output_buffer.len() {
        let dis = dis_spline.sample(i as f32).unwrap_or(1.0);
        output_buffer[i] = buffer[i] * dis;
    }
    
    output_buffer
}

fn smooth(
    buffer: &mut Vec<f32>,
    smoothing: usize,
    smoothing_size: usize,
) {
    if buffer.len() <= smoothing_size || smoothing_size == 0 {
        return;
    }
    for _ in 0..smoothing {
        for i in 0..buffer.len() {
            //let percentage: f32 = (buffer.len() - i) as f32 / buffer.len() as f32;
            //let smoothing_size: usize = (smoothing_size as f32 * percentage) as usize + 1;
            let mut y: f32 = 0.0;
            for x in 0..smoothing_size {
                if buffer.len() > i + x {
                    y += buffer[i+x];
                }
            }
            buffer[i] = y / smoothing_size as f32;
        }
        // remove parts that cannot be smoothed
        //buffer.drain(buffer.len() - 1 - smoothed..);
    }
}

// combines 2-dimensional buffer (Vec<Vec<f32>>) into a 1-dimensional one that has the average value of the 2D buffer
// EVERY 1D buffer of whole buffer MUST have the same length, but the current implementation guarantees this, considering the resolution stays the same
// if size changes you have to call 'Event::ClearBuffer'
#[allow(clippy::ptr_arg)]
pub fn merge_buffers(
    buffer: &Vec<Vec<f32>>, // EVERY 1D buffer of whole buffer MUST have the same length
) -> Vec<f32> {
    let mut smoothed_percentage: f32 = 0.0;
    let mut output_buffer: Vec<f32> = vec![0.0; buffer[0].len()];
    for (pos_z, z_buffer) in buffer.iter().enumerate() {
        // needed for weighting the Importance of earch z_buffer, more frequent -> more important
        // should decrease latency and increase overall responsiveness
        let percentage: f32 = (pos_z + 1) as f32 / buffer.len() as f32;
        smoothed_percentage += percentage;
        for (pos_x, value) in z_buffer.iter().enumerate() {
            if pos_x < output_buffer.len() {
                output_buffer[pos_x] += value * percentage;
            }
        }
    }

    for b in output_buffer.iter_mut() {
        *b /= smoothed_percentage;
    }

    output_buffer
}