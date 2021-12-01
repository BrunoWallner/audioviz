use crate::config::{ProcessorConfig, VolumeNormalisation};
use crate::config::Interpolation as ConfigInterpolation;

use rustfft::{num_complex::Complex, FftPlanner};
use splines::{Interpolation, Key, Spline};

/// Single Frequency
/// 
/// Multiple of these are stored in a Vector,
#[derive(Clone, Debug)]
pub struct Frequency {
    pub volume: f32,

    /// Actual frequency in hz, can range from 0 to `config.sample_rate` / 2
    /// 
    /// Accuracy can vary and is not guaranteed
    pub freq: f32,

    /// Relative position of single frequency in range (0..=1)
    /// 
    /// Used to make lower freqs occupy more space than higher ones, to mimic human hearing
    /// 
    /// Should not be Important, except when distributing freqs manually
    /// 
    /// To do this manually set `config.interpolation` equal to `Interpolation::None`
    pub position: f32,
}
impl Frequency {
    pub fn empty() -> Self {
        Frequency {volume: 0.0, freq: 0.0, position: 0.0}
    }
}

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

    pub fn compute_all(&mut self) {
        self.apodize();
        self.fft();
        self.distribute_volume();
        self.normalize_and_distribute();
    }

    pub fn apodize(&mut self) {
        let window = apodize::hanning_iter(self.raw_buffer.len()).collect::<Vec<f64>>();
        for i in 0..self.raw_buffer.len() {
            self.raw_buffer[i] *= window[i] as f32;
        }
    }

    // fft algorithm on raw_buffer
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
        self.raw_buffer = self.raw_buffer[0..(self.raw_buffer.len() as f32 * 0.5 ) as usize].to_vec();
    }

    // distributes volume on raw_buffer
    pub fn distribute_volume(&mut self) {
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

    // normalizes raw_buffer into freq_buffer
    pub fn normalize_and_distribute(&mut self) {
        let mut pos_index: Vec<(f32, f32)> = Vec::new(); // freq, volume

        for i in 0..self.raw_buffer.len() {
            // space normalisation and space distribution
            //let pos = self.normalized_pos(i as f32, self.buffer.len()) as usize;
            
            //let norm_offset: f32 = self.normalized_offset(i as f32, self.raw_buffer.len());
            //let norm_pos: f32 = self.normalized_position(i as f32, self.raw_buffer.len());

            let freq: f32 = ((i + 1) as f32 / self.raw_buffer.len() as f32) * (self.config.sample_rate as f32) / 2.0;
            pos_index.push((freq, self.raw_buffer[i].sqrt()));
        }

        //
        // applies offset and distribution to frequency position in self.buffer
        // that gets later applied on data request in audio_stream.rs
        // also applies normalisation
        //
        match &self.config.frequency_distribution {
            Some(distribution) => {
                let mut dis_pointer: f32 = 0.0;
                let mut abs_pointer: f32 = 0.0;
            
                let dis_spline = get_dis_spline(distribution.clone());
        
                for val in pos_index.iter(){
                    let freq = val.0;
                    //let norm_pos = val.1;
        
                    let eq_offset = get_dis_offset(&dis_spline, freq);
                    dis_pointer += eq_offset;
                    abs_pointer = dis_pointer;
        
                    let position = abs_pointer;
                    let freq = freq;
                    let volume = val.1 * self.config.volume;
        
                    self.freq_buffer.push(Frequency {
                        position,
                        freq,
                        volume,
                    });
                }
        
                // relative position in range (0..1)
                // with normalisation
                for freq in self.freq_buffer.iter_mut() {
                    freq.position /= abs_pointer;
                    freq.position = freq.position.powf(0.25);
                }
            }
            None => {
                for (i, val) in pos_index.iter().enumerate() {
                    let freq = val.0;
                    //let norm_pos = val.1;
                    
                    let volume = val.1 * self.config.volume;
                    //let position = i as f32 * norm_offset;

                    self.freq_buffer.push(Frequency {
                        position: i as f32,
                        freq,
                        volume,
                    });
                }

                // relative position in range (0..1)
                // with normalisation
                let freq_buf_len = self.freq_buffer.len() as f32;
                for freq in self.freq_buffer.iter_mut() {
                    freq.position /= freq_buf_len;
                    freq.position = freq.position.powf(0.25);
                }
            }
        }

        fn get_dis_spline(distribution: Vec<(usize, f32)>) -> Spline<f32, f32> {
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for freq_dis in distribution.iter() {
                points.push(Key::new(freq_dis.0 as f32, freq_dis.1, Interpolation::Linear));
            }
            Spline::from_vec(points)
        }
        fn get_dis_offset(spline: &Spline<f32, f32>, freq: f32) -> f32 {
            spline.clamped_sample(freq).unwrap_or(1.0)
        }
    }

    pub fn interpolate(&mut self) {
        // APPLIES POSITIONS TO FREQUENCIES and interpolation
        // VERY IMPORTANT
        self.freq_buffer = match self.config.interpolation {
            ConfigInterpolation::None => self.freq_buffer.clone(),
            ConfigInterpolation::Gaps => {
                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); self.freq_buffer.len()];
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
            },
            /*
            it seems like overlapping is ocurring in low freqs
            */
            ConfigInterpolation::Step => {
                let mut o_buf: Vec<Frequency> = vec![Frequency::empty(); self.freq_buffer.len()];
                let mut freqs = self.freq_buffer.iter().peekable();
                'filling: loop {
                    let freq: &Frequency = match freqs.next() {
                        Some(f) => f,
                        None => break 'filling,
                    };
                    
                    let start: usize = (freq.position * o_buf.len() as f32) as usize;
                    let end = 
                    (
                        match freqs.peek() {
                            Some(f) => f.position,
                            None => 1.0,
                        } * o_buf.len() as f32
                    ) as usize;
                    
                    for i in start..=end {
                        if o_buf.len() > i {
                            o_buf[i] = freq.clone();
                        }
                    }
                }

                o_buf
            }
            _ => self.freq_buffer.clone()
        };
    }

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
            if self.freq_buffer[self.freq_buffer.len() - (i + 1)].freq < self.config.frequency_bounds[1] as f32 {
                end = self.freq_buffer.len() - i;
                break;
            }
            i += 1;
        }

        // bounds
        self.freq_buffer = self.freq_buffer[start..end].to_vec();
    }

    #[allow(clippy::collapsible_if)]
    pub fn apply_resolution(&mut self) {
        let current_bars: f32 = self.freq_buffer.len() as f32;
        let resolution: f32 = match self.config.resolution {
            Some(res) => {
                res as f32 / current_bars
            }
            None => 1.0,
        };

        if resolution < 1.0 {
            let mut output_buffer: Vec<Frequency> =
                vec![Frequency::empty(); (self.freq_buffer.len() as f32 * resolution) as usize];

            let offset = output_buffer.len() as f32 / self.freq_buffer.len() as f32;
            for (i, freq) in self.freq_buffer.iter().enumerate() {
                let pos = (i as f32 * offset) as usize;

                // cannot be collapsed as clippy notes i think
                if pos < output_buffer.len() {
                    // crambling type
                    if output_buffer[pos].volume < freq.volume {
                        output_buffer[pos] = Frequency {volume: freq.volume, freq: freq.freq, position: freq.position};
                    }
                }
            }

            self.freq_buffer = output_buffer;
        }
    }

    /*
    fn normalized_position(&self, linear_pos: f32, buf_len: usize) -> f32 {
        //(buf_len as f32 / (linear_pos + 1.0) as f32).powf(0.5)
        //(buf_len as f32 / (linear_pos + 1.0) as f32).log(2.0)
        let offset = (linear_pos + 1.0) / buf_len as f32;
        buf_len as f32 * offset.sqrt()
    }
    */
}