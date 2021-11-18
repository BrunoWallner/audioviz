use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub smoothing_size: usize,
    pub smoothing_amount: usize,
    pub fft_resolution: usize,
    pub refresh_rate: usize,
    pub bar_count: usize,
    pub volume: f32,
    pub volume_normalisation: VolumeNormalisation,
    pub eq: Vec<( [usize; 2], f32 )>,
    pub gravity: Option<f32>,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            smoothing_size: 1,
            smoothing_amount: 1,
            fft_resolution: 1024 * 4,
            refresh_rate: 60,
            bar_count: 200,
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Linear(0.65),
            eq: vec![ ( [0, 100], 200.0 ), ( [100, 200], 100.0 ), ( [200, 2500], 200.0 ) ],
            gravity: Some(2.0),
        }
    }
}

// I know it can be replaced with Option<>, but I want to add things in the future
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VolumeNormalisation {
    None,
    Linear(f32),
}
