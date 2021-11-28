use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// neccessary so that the Audiostream knows what the hightest frequency is. (`sample_rate` / 2)
    pub sample_rate: u32,

    /// with higher resolution comes better precision, that is mostly needed for lower frequencies
    pub fft_resolution: usize,
    
    /// should be set to match fps of output, gravity will be affected, because I have not implemented delta-time
    pub refresh_rate: usize,

    /// range of frequencies
    pub frequency_bounds: [usize; 2],

    /// number of total frequencies in processed data
    pub bar_count: usize,
    pub volume: f32,

    /// to compensate for too loud low frequencies
    pub volume_normalisation: VolumeNormalisation,

    /// manually apply scale of frequencies
    /// 
    /// frequencies around 50hz have double the scale: `vec![ (50, 2.0) ]`
    /// 
    /// this can be applied to an infinite number of frequencies: `vec![ (20, 1.0), (500, 2.0), (5000, 0.5) ... ]`
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

    /// ```text
    ///           | |
    ///           | |
    ///       | | | |
    ///       | | | | | |
    ///       | | | | | |
    /// | | | | | | | | | |
    /// +++++++++++++++++++
    /// ```
    Step,

    /// ```text
    ///           | 
    ///         | | |  
    ///       | | | | 
    ///     | | | | | | | 
    ///   | | | | | | | | 
    /// | | | | | | | | | |
    /// +++++++++++++++++++
    /// ```
    Linear,

    /// ```text
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
