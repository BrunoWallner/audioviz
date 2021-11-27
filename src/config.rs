use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub sample_rate: u32,
    pub fft_resolution: usize,
    pub refresh_rate: usize,
    pub frequency_bounds: [usize; 2],
    pub bar_count: usize,
    pub volume: f32,
    pub volume_normalisation: VolumeNormalisation,
    pub frequency_distribution: Vec<(usize, f32)>,
    pub gravity: Option<f32>,
    pub interpolation: Interpolation,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            sample_rate: 44_100,
            fft_resolution: 1024 * 4,
            refresh_rate: 60,
            frequency_bounds: [30, 15000],
            bar_count: 200,
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Linear(0.65),
            frequency_distribution: vec![ (30, 3.0), (150, 4.0), (2000, 2.0), (5000, 1.0) ],
            gravity: Some(2.0),
            interpolation: Interpolation::Linear,
        }
    }
}

// I know it can be replaced with Option<>, but I want to add things in the future
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VolumeNormalisation {
    None,
    Linear(f32),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Interpolation {
    /// Not recommended
    /// 
    /// All frequencies are tightly packed together without space distribution
    /// 
    /// This will skip distribution, so you would have to do it manually
    None,

    /// ```
    ///           | |
    ///           | |
    ///       | | | |
    ///       | | | | | |
    ///       | | | | | |
    /// | | | | | | | | | |
    /// +++++++++++++++++++
    /// ```
    Step,

    /// ``` 
    ///           | 
    ///         | | |  
    ///       | | | | 
    ///     | | | | | | | 
    ///   | | | | | | | | 
    /// | | | | | | | | | |
    /// +++++++++++++++++++
    /// ```
    Linear,

    /// ```
    ///           |  
    ///           |  
    ///       |   |   | | 
    ///       |   |   | | 
    /// |     |   |   | | |
    /// +++++++++++++++++++
    /// ```
    /// The Gaps are empty Frequencies, (`Frequency::empty()`)
    Gaps,
}
