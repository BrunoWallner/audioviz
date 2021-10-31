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
}
impl Default for Config {
    fn default() -> Self {
        Config { 
            buffering: 10,
            smoothing_size: 5,
            smoothing_amount: 5,
            fft_resolution: 2048 * 3,
            refresh_rate: 60,
            bar_count: 200,
            max_frequency: 5_000,
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Exponentially,
            pre_fft_buffer_cutoff: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VolumeNormalisation {
    Manual(Vec<f32>),   // one good would look like this: [0.01, 0.1, 1.0, 1.0,  2.0]
    Linear,             // untested
    Exponentially,      // tested
}