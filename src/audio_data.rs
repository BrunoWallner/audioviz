use crate::config::{Config, VolumeNormalisation};
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
pub struct AudioData {
    config: Config,
    pub buffer: Vec<Frequency>,
}

impl AudioData {
    pub fn new(config: Config, data: &[f32]) -> Self {
        let buf: Vec<Frequency> = 
            data
                .into_iter()
                .map(|volume| Frequency {volume: *volume, freq: 0.0, position: 0.0})
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
        self.normalize_and_distribute();
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

    pub fn normalize_and_distribute(&mut self) {
        let mut pos_index: Vec<(f32, f32)> = Vec::new(); // freq, norm_offset

        for i in 0..self.buffer.len() {
            // space normalisation and space distribution
            //let pos = self.normalized_pos(i as f32, self.buffer.len()) as usize;
            let norm_offset: f32 = self.normalized_offset(i as f32, self.buffer.len());

            let freq: f32 = ((i + 1) as f32 / self.buffer.len() as f32) * (self.config.sample_rate as f32) / 2.0;
            pos_index.push((freq, norm_offset));
        }

        //
        // applies offset and distribution to frequency position in self.buffer
        // that gets later applied on data request in audio_stream.rs
        //
        let mut dis_pointer: f32 = 0.0;
        let mut abs_pointer: f32 = 0.0;
    
        let dis_spline = get_dis_spline(&self.config);

        for (i, val) in pos_index.iter().enumerate() {
            let freq = val.0;
            let norm_offset = val.1 as f32;

            let eq_offset = get_dis_offset(&dis_spline, freq);
            dis_pointer += eq_offset;
            abs_pointer = dis_pointer * norm_offset;

            self.buffer[i].position = abs_pointer;

            self.buffer[i].freq = freq;
            self.buffer[i].volume *= self.config.volume;
        }

        // relative position in range (0..1)
        for freq in self.buffer.iter_mut() {
            freq.position /= abs_pointer;
        }

        fn get_dis_spline(config: &Config) -> Spline<f32, f32> {
            let mut points: Vec<Key<f32, f32>> = Vec::new();
            for freq_dis in config.frequency_distribution.iter() {
                points.push(Key::new(freq_dis.0 as f32, freq_dis.1, Interpolation::Linear));
            }
            Spline::from_vec(points)
        }
        fn get_dis_offset(spline: &Spline<f32, f32>, freq: f32) -> f32 {
            spline.clamped_sample(freq).unwrap_or(1.0)
        }
    }

    fn normalized_offset(&self, linear_pos: f32, buf_len: usize) -> f32 {
        (buf_len as f32 / (linear_pos + 1.0) as f32).powf(0.5)
    }
}