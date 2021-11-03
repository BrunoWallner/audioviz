use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Config {
    pub buffering: usize,
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub fft_resolution: usize,
    pub refresh_rate: usize,
    pub bar_count: usize,
    pub max_frequency: usize,
    pub volume: f32,
    pub volume_normalisation: VolumeNormalisation,

    // higher means less latency but combined with high fft_resolution it will look bad
    // should be decreased with higher fft resolutions
    // MUST be in between 0 and 1
    pub pre_fft_buffer_cutoff: f32,

    // higher means more space for bass freqs
    // should be in between 0.1 and 1.0
    pub distribution: f32,
}
impl Default for Config {
    fn default() -> Self {
        Config { 
            buffering: 7,
            smoothing_size: 2,
            smoothing_amount: 2,
            fft_resolution: 1024 * 8,
            refresh_rate: 60,
            bar_count: 200,
            max_frequency: 5_000,
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Exponential,
            pre_fft_buffer_cutoff: 0.33,
            distribution: 0.5,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub enum VolumeNormalisation {
    Manual(Vec<f32>),   // one good would look like this: [0.01, 0.1, 1.0, 1.0,  2.0]
    Linear,
    Exponential,
}