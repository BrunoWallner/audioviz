#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub buffering: usize,
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub resolution: usize,
    pub refresh_rate: usize,
    pub frequency_scale_range: [usize; 2],
    pub frequency_scale_amount: usize,
    pub bar_reduction: usize,
}
impl Default for Config {
    fn default() -> Self {
        Config { 
            buffering: 10,
            smoothing_size: 15,
            smoothing_amount: 3,
            resolution: 3000,
            refresh_rate: 60,
            frequency_scale_range: [0, 3500],
            frequency_scale_amount: 2,
            bar_reduction: 2,
        }
    }
}