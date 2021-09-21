use rustfft::{num_complex::Complex, FftPlanner};
use crate::config::Config;

/// puts buffer into FFT alogrithm and applies filters and modifiers to it
pub fn convert_buffer(
    input_buffer: Vec<f32>,
    config: Config,
) -> Vec<f32> {
    let input_buffer = apodize(input_buffer);

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(input_buffer.len());

    let mut buffer: Vec<Complex<f32>> = Vec::new();
    for i in input_buffer.iter() {
        buffer.push(Complex {
            re: *i,
            im: 0.0,
        });
    }
    fft.process(&mut buffer[..]);

    let mut output_buffer: Vec<f32> = Vec::new();
    for i in buffer.iter() {
        output_buffer.push(i.norm())
    }

    // remove mirroring
    let output_buffer = output_buffer[0..(output_buffer.len() as f32 * 0.25) as usize].to_vec();

    let mut output_buffer = normalize(output_buffer);

    scale_frequencies(
        &mut output_buffer,
        config.frequency_scale_range,
        config.frequency_scale_amount
    );

    smooth(&mut output_buffer, config.smoothing_amount, config.smoothing_size);

    bar_reduction(&mut output_buffer, config.bar_reduction);

    output_buffer
}

fn apodize(buffer: Vec<f32>) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
}

fn scale_frequencies(buffer: &mut Vec<f32>, fav_freqs: [usize; 2], doubling: usize) {
    let mut doubled: usize = 0;
    let buffer_len = buffer.len();
    for i in 0..doubling {
        let start_percentage: f32 = fav_freqs[0] as f32 / 20_000.0;
        let end_percentage: f32 = fav_freqs[1] as f32 / 20_000.0;

        let start_pos: usize = (buffer_len as f32 * start_percentage) as usize;
        let end_pos: usize = (buffer_len as f32 * end_percentage) as usize;

        let mut normalized_start_pos: usize = ((buffer_len as f32 / start_pos as f32).sqrt() * start_pos as f32) as usize;
        normalized_start_pos = (normalized_start_pos as f32 * (1.0 - ( ( (i + 1) as f32 / doubling as f32) * 0.25))) as usize; // for smoothing edge between non scaled and scaled freqs

        let normalized_end_pos: usize = ((buffer_len as f32 / end_pos as f32).sqrt() * end_pos as f32) as usize + doubled;

        let mut position: usize = normalized_start_pos;
        for _ in normalized_start_pos..normalized_end_pos {
            if position < buffer.len() - 1 {
                let value: f32 = (buffer[position] + buffer[position + 1]) / 2.0;

                buffer.insert(position + 1, value);
                position += 2;
                doubled += 1;
            }
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn normalize(buffer: Vec<f32>) -> Vec<f32> {
    let buffer_len: usize = buffer.len();
    let mut output_buffer: Vec<f32> = vec![0.0; buffer_len];

    let mut start_pos: usize = 0;
    let mut end_pos: usize = 0;

    let mut pos_index: Vec<[usize; 2]> = Vec::new();

    for i in 0..buffer_len {
        let offset: f32 = (buffer_len as f32 / (i + 1) as f32).sqrt();
        if ((i as f32 * offset) as usize) < output_buffer.len() {
            // sets positions needed for future operations
            let pos: usize = (i as f32 * offset) as usize;
            start_pos = end_pos;
            end_pos = pos;
            pos_index.push([start_pos, end_pos]);

            // volume normalisation
            let volume_offset: f32 = (i + 1) as f32 / buffer_len as f32;
            let y = buffer[i] * volume_offset;

            if output_buffer[pos] < y {
                output_buffer[pos] = y;
            }
        }
        if end_pos - start_pos > 1 && (end_pos - 1) < output_buffer.len() {
            // filling
            for s_p in (start_pos + 1)..end_pos {
                let percentage: f32 = (s_p - start_pos) as f32 / ((end_pos - 1) - start_pos) as f32;

                let mut y: f32 = 0.0;
                //(output_buffer[s_p] * (1.0 - percentage) ) + (output_buffer[end_pos] * percentage);
                y += output_buffer[start_pos] * (1.0 - percentage);
                y += output_buffer[end_pos] * percentage;
                output_buffer[s_p] = y;
            }
        }
    }

    output_buffer
}

fn smooth(
    buffer: &mut Vec<f32>,
    smoothing: usize,
    smoothing_size: usize,
) {
    for _ in 0..smoothing {
        for i in 0..buffer.len() - smoothing_size as usize {
            // reduce smoothing for higher freqs
            let percentage: f32 = i as f32 / buffer.len() as f32;
            let smoothing_size = (smoothing_size as f32 * (1.5 - percentage.powf(2.0))) as u32;

            let mut y = 0.0;
            for x in 0..smoothing_size as usize {
                if buffer.len() > i + x {
                    y += buffer[i+x];
                }
            }
            buffer[i] = y / smoothing_size as f32;
        }
    }
}

fn bar_reduction(buffer: &mut Vec<f32>, bar_reduction: usize) {
    if bar_reduction == 0 {return}
    let mut position: usize = 0;

    'reducing: loop {
        // break if reached end of buffer
        if position + bar_reduction as usize >= buffer.len() {
            break 'reducing;
        }

        // smoothing of bars that are gonna be removed into the bar that stays
        let mut y: f32 = 0.0;
        for x in 0..bar_reduction as usize {
            y += buffer[position + x];
        }
        buffer[position] = y / bar_reduction as f32;

        if (position + bar_reduction as usize) < buffer.len() {
            buffer.drain(position..position + bar_reduction as usize);
        }

        position += 1;
    }

    // remove last parts of buffer that cannot easily be smoothed
    // with a large bar_reduction size this could heavily impact higher freqs 
    if buffer.len() > bar_reduction as usize {
        for _ in 0..bar_reduction {
            buffer.pop();
        }
    }
}

// combines 2-dimensional buffer (Vec<Vec<f32>>) into a 1-dimensional one that has the average value of the 2D buffer
// EVERY 1D buffer of whole buffer MUST have the same length, but the current implementation guarantees this, considering the resolution stays the same
pub fn combine_buffers(
    buffer: &Vec<Vec<f32>>, // EVERY 1D buffer of whole buffer MUST have the same length
) -> Vec<f32> {
    let mut smoothed_percentage: f32 = 0.0;
    let mut output_buffer: Vec<f32> = vec![0.0; buffer[0].len()];
    for (pos_z, z_buffer) in buffer.iter().enumerate() {
        // needed for weighting the Importance of earch z_buffer, more frequent -> more weight
        // should decrease latency
        let percentage: f32 = pos_z as f32 / buffer.len() as f32;
        //let percentage: f32 = 1.0;
        smoothed_percentage += percentage;
        for (pos_x, value) in z_buffer.iter().enumerate() {
            output_buffer[pos_x] += value * percentage;
        }
    }

    for b in output_buffer.iter_mut() {
        *b /= smoothed_percentage;
    }

    output_buffer
}