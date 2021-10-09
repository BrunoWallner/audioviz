use rustfft::{num_complex::Complex, FftPlanner};
use crate::config::Config;

/// puts buffer into FFT alogrithm and applies filters and modifiers to it
pub fn convert_buffer(
    input_buffer: &Vec<f32>,
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

    // max frequency
    let percentage: f32 = config.max_frequency as f32 / 20_000_f32;
    let output_buffer = output_buffer[0..(output_buffer.len() as f32 * percentage) as usize].to_vec();

    let mut output_buffer = normalize(output_buffer, config.volume);

    scale_frequencies(
        &mut output_buffer,
        config.frequency_scale_range,
        config.frequency_scale_amount,
        config.max_frequency,
    );

    smooth(&mut output_buffer, config.smoothing_amount, config.smoothing_size);

    bar_reduction(&mut output_buffer, config.density_reduction);

    output_buffer
}


fn apodize(buffer: &Vec<f32>) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
}

fn scale_frequencies(buffer: &mut Vec<f32>, fav_freqs: [usize; 2], doubling: usize, max_freqs: usize) {
    let mut doubled: usize = 0;
    let buffer_len = buffer.len();
    for _ in 0..doubling {
        let start_percentage: f32 = fav_freqs[0] as f32 / max_freqs as f32;
        let end_percentage: f32 = fav_freqs[1] as f32 / max_freqs as f32;

        let start_pos: f32 = buffer_len as f32 * start_percentage;
        let end_pos: f32 = buffer_len as f32 * end_percentage;

        let normalized_start_pos: usize = ((buffer_len as f32 / start_pos).sqrt() * start_pos) as usize;
        let normalized_end_pos: usize = ((buffer_len as f32 / end_pos).sqrt() * end_pos) as usize + doubled;

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
fn normalize(buffer: Vec<f32>, volume: f32) -> Vec<f32> {
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
            //let y = buffer[i] / offset.powi(2) * volume /* old and non linear method */
            let offset = (i+1) as f32 / 20_000.0;
            let y = buffer[i] * offset * volume;   /* new and linear method */

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
    if buffer.len() <= smoothing_size || smoothing_size == 0 {
        return;
    }
    for _ in 0..smoothing {
        for i in 0..buffer.len() {
            let percentage: f32 = (buffer.len() - i) as f32 / buffer.len() as f32;
            let smoothing_size: usize = (smoothing_size as f32 * percentage) as usize + 1;
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

fn bar_reduction(buffer: &mut Vec<f32>, bar_reduction: usize) {
    if bar_reduction == 0 {return}
    let mut position: usize = 0;

    'reducing: loop {
        // break if reached end of buffer
        if position >= buffer.len() {
            break 'reducing;
        }

        // smoothing of bars that are gonna be removed into the bar that remains
        let mut y: f32 = 0.0;
        let mut smoothed_amount: usize = 0;
        for x in 0..bar_reduction {
            if position + x < buffer.len() { 
                let value = buffer[position + x];
                y += value;
                smoothed_amount = x;
            }
        }
        buffer[position] = y / bar_reduction as f32;

        if smoothed_amount > 0 && position < buffer.len() && (position + smoothed_amount) < buffer.len() {
            buffer.drain(position..(position + smoothed_amount)); // causes panic when resolution changes and buffers get not cleared via event
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
// if size changes you have to call 'Event::ClearBuffer'
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