use crate::config::{Config, VolumeNormalisation};
use rustfft::{num_complex::Complex, FftPlanner};
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
        self.apodize();
        self.fft();
        self.distribute_volume();
        self.normalize_and_eq();
        self.smooth();
        //self.cut_off();
        self.apply_bar_count();
    }

    pub fn apodize(&mut self) {
        let window = apodize::hanning_iter(self.buffer.len()).collect::<Vec<f64>>();

        let mut output_buffer: Vec<f32> = Vec::new();

        for i in 0..self.buffer.len() {
            output_buffer.push(window[i] as f32 * self.buffer[i]);
        }
        self.buffer = output_buffer
    }

    pub fn fft(&mut self) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.buffer.len());

        let mut buffer: Vec<Complex<f32>> = Vec::new();
        for i in self.buffer.iter() {
            buffer.push(Complex { re: *i, im: *i });
        }
        fft.process(&mut buffer[..]);

        for (i, val) in buffer.iter().enumerate() {
            self.buffer[i] = val.norm();
        }
        // remove mirroring
        self.buffer = self.buffer[0..(self.buffer.len() as f32 * 0.5 ) as usize].to_vec();
    }

    pub fn distribute_volume(&mut self) {
        match &self.config.volume_normalisation {
            VolumeNormalisation::None => (),
            VolumeNormalisation::Linear(v) => {
                for i in 0..self.buffer.len() {
                    let percentage: f32 = i as f32 / self.buffer.len() as f32;
                    self.buffer[i] *= percentage.powf(*v);
                }
            }
        }
    }

    pub fn normalize_and_eq(&mut self) {
        let mut pos_index: Vec<(f32, f32, f32)> = Vec::new(); // freq, norm_offset, y_value

        for i in 0..self.buffer.len() {
            // space normalisation and space distribution
            //let pos = self.normalized_pos(i as f32, self.buffer.len()) as usize;
            let norm_offset: f32 = self.normalized_offset(i as f32, self.buffer.len());
            let y = self.buffer[i] * 0.05;

            let freq: f32 = ((i + 1) as f32 / self.buffer.len() as f32) * (self.config.sample_rate as f32) / 2.0;
            pos_index.push((freq, norm_offset, y));
        }

        // Interpolation and eq
        let mut points: Vec<Key<f32, f32>> = Vec::new();
        let mut eq_pointer: f32 = 0.0;
        let mut abs_pointer: f32 = 0.0;
    
        let eq_spline = get_eq_spline(&self.config);

        for val in pos_index.iter() {
            let freq = val.0;
            let norm_offset = val.1 as f32;
            let y = val.2 * self.config.volume;

            // freq bound cuttoff bc of if statements
            if (self.config.frequency_bounds[0] as f32) < freq {
                if (self.config.frequency_bounds[1] as f32) > freq {
                    let eq_offset = get_eq_offset(&eq_spline, freq);
                    eq_pointer += eq_offset;
                    abs_pointer = eq_pointer * norm_offset;

                    points.push(Key::new(abs_pointer, y, Interpolation::Linear));
                }
            }
        }

        let spline = Spline::from_vec(points);

        self.buffer.drain(..);

        for i in 0..(abs_pointer as usize) {
            match spline.sample(i as f32) {
                Some(v) => self.buffer.push(v),
                None => (),
            }
        }

        fn get_eq_spline(config: &Config) -> Spline<f32, f32> {
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for eq in config.eq.iter() {
                points.push(Key::new(eq.0 as f32, eq.1, Interpolation::Linear));
            }
            Spline::from_vec(points)
        }
        fn get_eq_offset(spline: &Spline<f32, f32>, freq: f32) -> f32 {
            spline.clamped_sample(freq).unwrap_or(1.0)
        }
    }

    pub fn smooth(&mut self) {
        if !(self.buffer.len() <= self.config.smoothing_size || self.config.smoothing_size == 0) {
            for _ in 0..self.config.smoothing_amount {
                for i in 0..self.buffer.len() {
                    // smoothing size drop for higher freqs
                    let percentage: f32 = (self.buffer.len() - i) as f32 / self.buffer.len() as f32;
                    let smoothing_size: usize =
                        (self.config.smoothing_size as f32 * percentage) as usize + 1;

                    let mut y: f32 = 0.0;
                    for x in 0..smoothing_size {
                        if self.buffer.len() > i + x {
                            y += self.buffer[i + x];
                        }
                    }
                    self.buffer[i] = y / smoothing_size as f32;
                }
            }
        }
    }

    #[allow(clippy::collapsible_if)]
    pub fn apply_bar_count(&mut self) {
        let current_bars: f32 = self.buffer.len() as f32;
        let resolution: f32 = self.config.bar_count as f32 / current_bars;

        let mut output_buffer: Vec<f32> =
            vec![0.0; (self.buffer.len() as f32 * resolution) as usize];

        if resolution > 1.0 {
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for (i, val) in self.buffer.iter().enumerate() {
                points.push(Key::new(i as f32 * resolution, *val, Interpolation::Linear));
            }

            let spline = Spline::from_vec(points);

            for (i, val) in output_buffer.iter_mut().enumerate() {
                let v = spline.clamped_sample(i as f32).unwrap_or(0.0);
                *val = v;
            }

            self.buffer = output_buffer;
        } else if resolution < 1.0 {
            let offset = output_buffer.len() as f32 / self.buffer.len() as f32;
            for (i, val) in self.buffer.iter().enumerate() {
                let pos = (i as f32 * offset) as usize;

                // cannot be collapsed as clippy notes i think
                if pos < output_buffer.len() {
                    // crambling type
                    if output_buffer[pos] < *val {
                        output_buffer[pos] = *val;
                    }

                    // smoothing type
                    //output_buffer[pos] = (output_buffer[pos] + *val) * 0.5;
                }
            }

            self.buffer = output_buffer;
        } else {
        }
    }

    #[inline]
    fn normalized_offset(&self, linear_pos: f32, buf_len: usize) -> f32 {
        (buf_len as f32 / (linear_pos + 1.0) as f32).powf(0.5)
    }
}