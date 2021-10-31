use rustfft::{num_complex::Complex, FftPlanner};
use crate::config::{Config, VolumeNormalisation};
use splines::{Interpolation, Key, Spline};

pub struct AudioData {
    config: Config,
    pub buffer: Vec<f32>,
}

impl AudioData {
    pub fn new(config: Config, data: &[f32]) -> Self {
        AudioData {
            config,
            buffer: data.to_vec(),
        }
    }

    pub fn compute_all(&mut self) {
        self.fft();
        self.normalize();
        self.distribute_volume();
        self.smooth();
        self.cut_off();
        self.apply_bar_count();
    }

    pub fn fft(&mut self) {
        let i_buf = apodize(&self.buffer[..]);

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(i_buf.len());
    
        let mut buffer: Vec<Complex<f32>> = Vec::new();
        for i in i_buf.iter() {
            buffer.push(Complex {
                re: *i,
                im: *i,
            });
        }
        fft.process(&mut buffer[..]);

        for (i, val) in buffer.iter().enumerate() {
            self.buffer[i] = val.norm();
        }
        // remove mirroring
        self.buffer = self.buffer[0..(self.buffer.len() as f32 * 0.25) as usize].to_vec();
    }

    pub fn distribute_volume(&mut self) {
        match &self.config.volume_normalisation {
            VolumeNormalisation::Manual(distribution) => {
                let mut dis_points: Vec<Key<f32, f32>> = Vec::new();
                let step = self.buffer.len() as f32 / (distribution.len() - 1) as f32;
            
                for (i, val) in distribution.iter().enumerate() {
                    dis_points.push(Key::new(i as f32 * step, *val, Interpolation::Linear));
                }
                let dis_spline = Spline::from_vec(dis_points);
            
                for i in 0..self.buffer.len() {
                    let offset: f32 = (self.buffer.len() as f32 / (i + 1) as f32).powf(0.5);
                    let pos: f32 = i as f32 * offset;
                    let dis = dis_spline.sample(pos).unwrap_or(1.0);
            
                    self.buffer[i] *= dis;
                }
            }
            VolumeNormalisation::Linear => {
                let buf_len = self.buffer.len();
                for (i, val) in self.buffer.iter_mut().enumerate() {
                    let percentage: f32 = i as f32 / buf_len as f32;
                    *val *= percentage * 2.0;
                }
            }
            VolumeNormalisation::Exponentially => {
                let buf_len = self.buffer.len();
                for (i, val) in self.buffer.iter_mut().enumerate() {
                    let offset: f32 = (buf_len as f32 / (i + 1) as f32).powf(0.5);
                    let pos = i as f32 * offset;
                    let volume_offset: f32 = (buf_len as f32 / (pos.powi(2) + 1.0)).powf(0.5);

                    *val /= volume_offset.powi(3)  * 10.0_f32.powi(4); // las bit is needed so that value is between 0 and 1
                }
            }
        }
    }

    pub fn normalize(&mut self) {
        let mut pos_index: Vec<(usize, f32)> = Vec::new();
    
        for i in 0..self.buffer.len() {
            let offset: f32 = (self.buffer.len() as f32 / (i + 1) as f32).powf(0.5);
    
            if ((i as f32 * offset) as usize) < self.buffer.len() {
                // space normalisation and space distribution
                let pos = (i as f32 * offset) as usize;
    
                // volume normalisation
                //let volume_offset: f32 = (output_buffer.len() as f32 / (pos + 1) as f32).powf(0.5);
                //let y = buffer[i] / volume_offset.powi(3) * 0.01;
                let y = self.buffer[i] * 0.01;
    
                pos_index.push( (pos, y) );
            }
        }
    
        // Interpolation
        let mut points: Vec<Key<f32, f32>> = Vec::new();
        for val in pos_index.iter() {
            let x = val.0 as f32;
            let y = val.1 * self.config.volume;
            points.push(Key::new(x, y, Interpolation::Linear));
        }
    
        let spline = Spline::from_vec(points);
    
        for i in 0..self.buffer.len() {
            let v = spline.clamped_sample(i as f32).unwrap_or(0.0);
            self.buffer[i] = v;
        }
    }

    #[inline]
    pub fn cut_off(&mut self) {
        let percentage: f32 = self.config.max_frequency as f32 / 22_500_f32;
        let pos = self.normalized_pos(
            (self.buffer.len() as f32 * percentage) as usize 
        );
        self.buffer = self.buffer[0..pos].to_vec();
    }

    pub fn smooth(&mut self) {
        if !(self.buffer.len() <= self.config.smoothing_size || self.config.smoothing_size == 0) {
            for _ in 0..self.config.smoothing_amount {
                for i in 0..self.buffer.len() {
                    //let percentage: f32 = (buffer.len() - i) as f32 / buffer.len() as f32;
                    //let smoothing_size: usize = (smoothing_size as f32 * percentage) as usize + 1;
                    let mut y: f32 = 0.0;
                    for x in 0..self.config.smoothing_size {
                        if self.buffer.len() > i + x {
                            y += self.buffer[i+x];
                        }
                    }
                    self.buffer[i] = y / self.config.smoothing_size as f32;
                }
                // remove parts that cannot be smoothed
                //buffer.drain(buffer.len() - 1 - smoothed..);
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    pub fn apply_bar_count(&mut self) {
        let lin_percentage: f32 = self.config.max_frequency as f32 / 22_500_f32;
        let norm_pos = self.normalized_pos(
            (self.buffer.len() as f32 * lin_percentage) as usize 
        );

        let current_bars: f32 = self.config.fft_resolution as f32 * 0.25 * (norm_pos as f32 / self.buffer.len() as f32);
        let resolution: f32 = self.config.bar_count as f32 / current_bars;

        let mut output_buffer: Vec<f32> = vec![0.0; (self.buffer.len() as f32 * resolution) as usize];

        if resolution > 1.0 {    
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for (i, val) in self.buffer.iter().enumerate() {
                points.push(Key::new(i  as f32 * resolution, *val, Interpolation::Linear));
            }
        
            let spline = Spline::from_vec(points);
        
            for (i, val) in output_buffer.iter_mut().enumerate() {
                let v = spline.clamped_sample(i as f32).unwrap_or(0.0);
                *val = v;
            }
        
            self.buffer = output_buffer;
        }
        else if resolution < 1.0 {
            let offset = output_buffer.len() as f32 / self.buffer.len() as f32;
            for (i, val) in self.buffer.iter().enumerate() {
                let pos = (i as f32 * offset) as usize;

                // cannot be collapsed
                if pos < output_buffer.len() {
                    if output_buffer[pos] < *val {
                        output_buffer[pos] = *val;
                    }
                }
            }
    
            self.buffer = output_buffer;
        }
        else {
        } 
    }

    fn normalized_pos(&self, linear_pos: usize) -> usize {
        let offset: f32 = (self.buffer.len() as f32 / (linear_pos + 1) as f32).powf(0.5);
        (linear_pos as f32 * offset) as usize
    }
}

fn apodize(buffer: &[f32]) -> Vec<f32> {
    let window = apodize::hanning_iter(buffer.len()).collect::<Vec<f64>>();

    let mut output_buffer: Vec<f32> = Vec::new();

    for i in 0..buffer.len() {
        output_buffer.push(window[i] as f32 * buffer[i]);
    }
    output_buffer
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
