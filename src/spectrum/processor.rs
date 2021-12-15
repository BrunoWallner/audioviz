use rustfft::{num_complex::Complex, FftPlanner};
use splines::{Interpolation, Key, Spline};

use crate::spectrum::config::Interpolation as ConfigInterpolation;
use crate::spectrum::config::{ProcessorConfig, VolumeNormalisation, SpaceNormalisation};

use crate::spectrum::Frequency;

/// struct that deals with processing for spectralized output with the help of Fast Fourier Transform
#[derive(Clone, Debug)]
pub struct Processor {
    config: ProcessorConfig,
    pub raw_buffer: Vec<f32>,
    pub freq_buffer: Vec<Frequency>,
}

impl Processor {
    pub fn from_raw_data(config: ProcessorConfig, data: Vec<f32>) -> Self {
        let freq_buf_cap: usize = data.len() / 2;
        Processor {
            config,
            raw_buffer: data,
            freq_buffer: Vec::with_capacity(freq_buf_cap),
        }
    }
    pub fn from_frequencies(config: ProcessorConfig, freqs: Vec<Frequency>) -> Self {
        Processor {
            config,
            raw_buffer: Vec::new(),
            freq_buffer: freqs,
        }
    }

    /// process everything in recommended order
    pub fn compute_all(&mut self) {
        self.apodize();
        self.fft();
        self.normalize_frequency_volume();

        self.raw_to_freq_buffer();
        self.normalize_frequency_position();
        self.distribute_frequency_position();
        self.bound_frequencies();
        self.interpolate();
    }

    pub fn apodize(&mut self) {
        let window = apodize::hanning_iter(self.raw_buffer.len()).collect::<Vec<f64>>();
        for (i, value) in self.raw_buffer.iter_mut().enumerate() {
            *value *= window[i] as f32;
        }
    }

    /// processes fft algorithm on `raw_buffer`
    pub fn fft(&mut self) {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(self.raw_buffer.len());

        let mut buffer: Vec<Complex<f32>> = Vec::new();
        for vol in self.raw_buffer.iter() {
            buffer.push(Complex { re: *vol, im: 0.0 });
        }
        fft.process(&mut buffer[..]);

        for (i, val) in buffer.iter().enumerate() {
            self.raw_buffer[i] = val.norm();
        }
        // remove mirroring
        self.raw_buffer =
            self.raw_buffer[0..(self.raw_buffer.len() as f32 * 0.5) as usize].to_vec();
    }

    /// normalizes volume on `raw_buffer` so that higher frequencies are louder
    pub fn normalize_frequency_volume(&mut self) {
        match &self.config.volume_normalisation {
            VolumeNormalisation::None => (),
            VolumeNormalisation::Linear(v) => {
                for i in 0..self.raw_buffer.len() {
                    let percentage: f32 = i as f32 / self.raw_buffer.len() as f32;
                    self.raw_buffer[i] *= percentage.powf(*v);
                }
            }
        }
    }

    /// manual position distribution on `freq_buffer`
    pub fn distribute_frequency_position(&mut self) {
        if let Some(distribution) = &self.config.manual_position_distribution {
            let dis_spline = get_dis_spline(distribution);

            let freq_buf_len: usize = self.freq_buffer.len();
            let mut last_position: f32 = 0.0;
            let mut pointer_pos: f32 = 0.0;
            for (i, val) in self.freq_buffer.iter_mut().enumerate() {
                let percentage: f32 = (i + 1) as f32 / freq_buf_len as f32;
                let freq: f32 = percentage * (self.config.sample_rate as f32 / 2.0);
                let offset = dis_spline.clamped_sample(freq).unwrap_or(1.0);

                let diff = val.position - last_position;

                pointer_pos += diff * offset;
                last_position = val.position;
                val.position = pointer_pos;
            }

            // makes sure that position of every frequency is <= 1.0
            let max_pos = self.freq_buffer[self.freq_buffer.len() - 1].position;
            for freq in self.freq_buffer.iter_mut() {
                freq.position /= max_pos;
            }
        }

        #[allow(clippy::ptr_arg)]
        fn get_dis_spline(distribution: &Vec<(usize, f32)>) -> Spline<f32, f32> {
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for freq_dis in distribution.iter() {
                points.push(Key::new(
                    freq_dis.0 as f32,
                    freq_dis.1,
                    Interpolation::Linear,
                ));
            }
            Spline::from_vec(points)
        }
    }

    /// populates the `freq_buffer` and applies volume
    pub fn raw_to_freq_buffer(&mut self) {
        for (i, val) in self.raw_buffer.iter().enumerate() {
            let percentage: f32 = (i + 1) as f32 / self.raw_buffer.len() as f32;
            self.freq_buffer.push(Frequency {
                volume: *val * self.config.volume,
                position: percentage,
                freq: percentage * (self.config.sample_rate as f32 / 2.0),
            });
        }
    }

    pub fn normalize_frequency_position(&mut self) {
        match self.config.position_normalisation {
            SpaceNormalisation::Exponential(exp) => {
                for freq in self.freq_buffer.iter_mut() {
                    freq.position = freq.position.powf(exp);
                } 
            }
            SpaceNormalisation::Harmonic => {
                let mut pos: f32 = 0.0;
                for (i, freq) in self.freq_buffer.iter_mut().enumerate() {
                    freq.position = pos;
                    pos += 1.0 / (i + 1) as f32;
                }

                // last freq must have position of 1.0
                let max_pos = match self.freq_buffer.last() {
                    Some(f) => f.position,
                    None => 1.0
                };
                for freq in self.freq_buffer.iter_mut() {
                    freq.position *= 1.0 / max_pos;
                }
            }
        }
    }

    /// applies the position of frequencies in `freq_buffer`, interpolates the gaps and applies resolution
    pub fn interpolate(&mut self) {
        // APPLIES POSITIONS TO FREQUENCIES and interpolation
        // VERY IMPORTANT
        let resolution = match self.config.resolution {
            Some(res) => res,
            None => self.freq_buffer.len(),
        };
        self.freq_buffer = match self.config.interpolation {
            ConfigInterpolation::None => self.freq_buffer.clone(),
            ConfigInterpolation::Gaps => {
                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); resolution];
                for freq in self.freq_buffer.iter() {
                    let abs_pos = (o_buf.len() as f32 * freq.position) as usize;
                    if o_buf.len() > abs_pos {
                        // louder freqs are more important and shall not be overwritten by others
                        if freq.volume > o_buf[abs_pos].volume {
                            o_buf[abs_pos] = freq.clone();
                        }
                    }
                }
                o_buf
            }
            /*
            it seems like overlapping is ocurring in low freqs
            */
            ConfigInterpolation::Step => {
                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); resolution];
                let mut freqs = self.freq_buffer.iter().peekable();

                'filling: loop {
                    let freq: &Frequency = match freqs.next() {
                        Some(f) => f,
                        None => break 'filling,
                    };

                    let start: usize = (freq.position * o_buf.len() as f32) as usize;
                    let end = (match freqs.peek() {
                        Some(f) => f.position,
                        None => 1.0,
                    } * o_buf.len() as f32) as usize;

                    for i in start..=end {
                        if o_buf.len() > i && o_buf[i].volume < freq.volume {
                            o_buf[i] = freq.clone();
                        }
                    }
                }

                o_buf
            }
            ConfigInterpolation::Linear => {
                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); resolution];
                let mut freqs = self.freq_buffer.iter().peekable();
                'interpolating: loop {
                    let start_freq: &Frequency = match freqs.next() {
                        Some(f) => f,
                        None => break 'interpolating,
                    };

                    let start: usize = (start_freq.position * o_buf.len() as f32) as usize;
                    let end_freq = match freqs.peek() {
                        Some(f) => f,
                        None => break 'interpolating,
                    };
                    let end: usize = (end_freq.position * o_buf.len() as f32) as usize;

                    if start < resolution && end < resolution {
                        for i in start..=end {
                            let pos: usize = i - start;
                            let gap_size = end - start;
                            let mut percentage: f32 = pos as f32 / gap_size as f32;
                            if percentage.is_nan() {percentage = 0.5}

                            // interpolation
                            let volume: f32 = (start_freq.volume * (1.0 - percentage))
                                + (end_freq.volume * percentage);
                            let position: f32 = (start_freq.position * (1.0 - percentage))
                                + (end_freq.position * percentage);
                            let freq: f32 = (start_freq.freq * (1.0 - percentage))
                                + (end_freq.freq * percentage);

                            if o_buf.len() > i && o_buf[i].volume < volume {
                                o_buf[i] = Frequency {
                                    volume,
                                    position,
                                    freq,
                                };
                            }
                        }
                    }
                }
                o_buf
            }
        };
    }

    /// applies frequency boundaries
    // I am not proud of it but it works
    pub fn bound_frequencies(&mut self) {
        // determines start pos
        let mut start: usize = 0;
        let mut i: usize = 0;
        loop {
            if i >= self.freq_buffer.len() {
                break;
            }
            if self.freq_buffer[i].freq > self.config.frequency_bounds[0] as f32 {
                start = i;
                break;
            }
            i += 1;
        }

        // determines end pos
        let mut end: usize = self.freq_buffer.len();
        let mut i: usize = 0;
        loop {
            if i >= self.freq_buffer.len() {
                break;
            }
            if self.freq_buffer[self.freq_buffer.len() - (i + 1)].freq
                < self.config.frequency_bounds[1] as f32
            {
                end = self.freq_buffer.len() - i;
                break;
            }
            i += 1;
        }

        // bounds
        let mut bound_buff = self.freq_buffer[start..end].to_vec();
        if !bound_buff.is_empty() {
            // fix for first and last frequency's position not being 0 and 1
            let start_pos: f32 = bound_buff[0].position;
            let end_pos: f32 = bound_buff[bound_buff.len() - 1].position - start_pos;
            let end_pos_offset: f32 = 1.0 / end_pos;

            for freq in bound_buff.iter_mut() {
                freq.position -= start_pos;
                freq.position *= end_pos_offset;
            }

            self.freq_buffer = bound_buff;
        }
    }
}
