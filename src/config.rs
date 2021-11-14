use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Config {
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub fft_resolution: usize,
    pub refresh_rate: usize,
    pub bar_count: usize,
    pub frequency_bounds: [usize; 2],
    pub volume: f32,
    pub volume_normalisation: VolumeNormalisation,

    // higher means less latency but combined with high fft_resolution it will look bad
    // should be decreased with higher fft resolutions
    // MUST be in between 0 and 1
    pub pre_fft_buffer_cutoff: f32,

    // higher means more space for bass freqs
    // should be in between 0.1 and 1.0
    pub distribution: Vec<f32>,

    pub gravity: Option<f32>,
}
impl Default for Config {
    fn default() -> Self {
        Config { 
            smoothing_size: 10,
            smoothing_amount: 10,
            fft_resolution: 1024 * 8,
            refresh_rate: 60,
            bar_count: 200,
            frequency_bounds: [30, 5000],
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Linear(0.85),
            pre_fft_buffer_cutoff: 0.33,
            distribution: vec![1.0, 1.0, 1.0, 1.0],
            gravity: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub enum VolumeNormalisation {
    None,
    Linear(f32),
}