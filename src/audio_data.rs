use crate::config::{Config, VolumeNormalisation};
use rustfft::{num_complex::Complex, FftPlanner};
use splines::{Interpolation, Key, Spline};

#[derive(Clone, Debug)]
pub struct Frequency {
    pub volume: f32,
    pub freq: f32,
}
impl Frequency {
    pub fn empty() -> Self {
        Frequency {volume: 0.0, freq: 0.0}
    }
}

#[derive(Clone, Debug)]
pub struct AudioData {
    config: Config,
    pub buffer: Vec<Frequency>,
}

impl AudioData {
    pub fn new(config: Config, data: &[f32]) -> Self {
        let buf: Vec<Frequency> = 
            data
                .into_iter()
                .map(|volume| Frequency {volume: *volume, freq: 0.0})
                .collect();

        AudioData {
            config,
            buffer: buf,
        }
    }

    pub fn compute_all(&mut self) {
        self.apodize();
        self.fft();
        self.distribute_volume();
        self.normalize_and_eq();
        self.apply_bar_count();
    }

    pub fn apodize(&mut self) {
        let window = apodize::hanning_iter(self.buffer.len()).collect::<Vec<f64>>();
        for i in 0..self.buffer.len() {
            self.buffer[i].volume *= window[i] as f32;
        }
    }

    pub fn fft(&mut self) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.buffer.len());

        let mut buffer: Vec<Complex<f32>> = Vec::new();
        for freq in self.buffer.iter() {
            buffer.push(Complex { re: freq.volume, im: 0.0 });
        }
        fft.process(&mut buffer[..]);

        for (i, val) in buffer.iter().enumerate() {
            self.buffer[i].volume = val.norm();
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
                    self.buffer[i].volume *= percentage.powf(*v);
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
            let y = self.buffer[i].volume * 0.05;

            let freq: f32 = ((i + 1) as f32 / self.buffer.len() as f32) * (self.config.sample_rate as f32) / 2.0;
            pos_index.push((freq, norm_offset, y));
        }

        // Interpolation and eq
        let mut vol_points: Vec<Key<f32, f32>> = Vec::new();
        let mut freq_points: Vec<Key<f32, f32>> = Vec::new();

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

                    vol_points.push(Key::new(abs_pointer, y, Interpolation::Linear));
                    freq_points.push(Key::new(abs_pointer, freq, Interpolation::Linear));
                }
            }
        }

        let vol_spline = Spline::from_vec(vol_points);
        let freq_spline = Spline::from_vec(freq_points);

        self.buffer.drain(..);

        for i in 0..(abs_pointer as usize) {
            match vol_spline.sample(i as f32) {
                Some(volume) => {
                    match freq_spline.sample(i as f32) {
                        Some(freq) => {
                            self.buffer.push(Frequency {volume, freq});
                        }
                        None => (),
                    }
                },
                None => (),
            }
        }

        fn get_eq_spline(config: &Config) -> Spline<f32, f32> {
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for freq_dis in config.frequency_distribution.iter() {
                points.push(Key::new(freq_dis.0 as f32, freq_dis.1, Interpolation::Linear));
            }
            Spline::from_vec(points)
        }
        fn get_eq_offset(spline: &Spline<f32, f32>, freq: f32) -> f32 {
            spline.clamped_sample(freq).unwrap_or(1.0)
        }
    }

    #[allow(clippy::collapsible_if)]
    pub fn apply_bar_count(&mut self) {
        let current_bars: f32 = self.buffer.len() as f32;
        let resolution: f32 = self.config.bar_count as f32 / current_bars;

        let mut output_buffer: Vec<Frequency> =
            vec![Frequency {volume: 0.0, freq: 0.0}; (self.buffer.len() as f32 * resolution) as usize];

        if resolution < 1.0 {
            let offset = output_buffer.len() as f32 / self.buffer.len() as f32;
            for (i, freq) in self.buffer.iter().enumerate() {
                let pos = (i as f32 * offset) as usize;

                // cannot be collapsed as clippy notes i think
                if pos < output_buffer.len() {
                    // crambling type
                    if output_buffer[pos].volume < freq.volume {
                        output_buffer[pos] = Frequency {volume: freq.volume, freq: freq.freq};
                    }
                }
            }

            self.buffer = output_buffer;
        }
    }

    fn normalized_offset(&self, linear_pos: f32, buf_len: usize) -> f32 {
        (buf_len as f32 / (linear_pos + 1.0) as f32).powf(0.5)
    }
}