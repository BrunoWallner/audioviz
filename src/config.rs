#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub buffering: usize,
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub resolution: usize,
    pub refresh_rate: usize,
    pub frequency_scale_range: [usize; 2],
    pub frequency_scale_amount: usize,
    pub density_reduction: usize,
    pub max_frequency: usize,
    pub volume: f32,
}
impl Default for Config {
    fn default() -> Self {
        Config { 
            buffering: 5,
            smoothing_size: 5,
            smoothing_amount: 5,
            resolution: 3000,
            refresh_rate: 60,
            frequency_scale_range: [0, 3500],
            frequency_scale_amount: 3,
            density_reduction: 10,
            max_frequency: 20_000,
            volume: 1.0,
        }
    }
}