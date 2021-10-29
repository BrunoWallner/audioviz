#[derive(Debug, Clone)]
pub struct Config {
    pub buffering: usize,
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub fft_resolution: usize,
    pub refresh_rate: usize,
    pub resolution: f32,
    pub max_frequency: usize,
    pub volume: f32,
    pub volume_normalisation: VolumeNormalisation,
}
impl Default for Config {
    fn default() -> Self {
        Config { 
            buffering: 6,
            smoothing_size: 6,
            smoothing_amount: 5,
            fft_resolution: 2048 * 3,
            refresh_rate: 60,
            resolution: 1.0,
            max_frequency: 20_000,
            volume: 100.0,
            volume_normalisation: VolumeNormalisation::Exponentially,
            //volume_normalisation: VolumeNormalisation::Manual(vec![1.0])
        }
    }
}

#[derive(Debug, Clone)]
pub enum VolumeNormalisation {
    Manual(Vec<f32>),   // untested
    Linear,             // untested
    Exponentially,      // tested
}