#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// I know it can be replaced with Option<>, but I want to add things in the future
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VolumeNormalisation {
    None,
    Harmonic
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum PositionNormalisation {
    Linear,
    Exponential,
    Harmonic,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Interpolation {
    /// Not recommended
    ///
    /// All frequencies are tightly packed together without space distribution applied
    ///
    /// you would have manually apply positions of frequencies
    /// ```text
    ///     |
    ///     |
    ///   | |
    ///   | | | |
    ///   | | | |
    /// | | | | | |
    /// ++++++++++++
    ///
    /// ```
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

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ProcessorConfig {
    /// neccessary so that the Audiostream knows what the hightest frequency is. (`sample_rate` / 2)
    pub sample_rate: u32,

    /// range of frequencies
    pub frequency_bounds: [usize; 2],

    /// number of total frequencies in processed data, None to disable downscaling
    ///
    /// max value is length of `input buffer` of raw data / 2
    pub resolution: Option<usize>,

    pub volume: f32,

    /// to even volume of low and high frequencies
    pub volume_normalisation: VolumeNormalisation,

    /// to mimic human hearing
    pub position_normalisation: PositionNormalisation,

    /// manually apply scale of frequencies
    ///
    /// frequencies around 50hz have double the scale: `vec![ (0, 1.0), (50, 2.0), (20000, 1.0) ]`
    ///
    /// this can be applied to an infinite number of frequencies: `vec![ (20, 1.0), (500, 2.0), (5000, 0.5) ... ]`
    pub manual_position_distribution: Option<Vec<(usize, f32)>>,

    pub interpolation: Interpolation,
}
impl Default for ProcessorConfig {
    fn default() -> Self {
        ProcessorConfig {
            sample_rate: 44_100,
            frequency_bounds: [40, 20000],
            resolution: None,
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Harmonic,
            position_normalisation: PositionNormalisation::Harmonic,
            manual_position_distribution: None,
            interpolation: Interpolation::Step,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamConfig {
    pub processor: ProcessorConfig,

    /// with higher resolution comes better precision, that is mostly needed for lower frequencies
    pub fft_resolution: usize,

    /// should be set to match fps of output, gravity will be affected, because I have not implemented delta-time
    pub refresh_rate: usize,

    pub gravity: Option<f32>,
}
impl Default for StreamConfig {
    fn default() -> Self {
        StreamConfig {
            processor: ProcessorConfig::default(),
            fft_resolution: 1024 * 4,
            refresh_rate: 60,
            gravity: Some(2.0),
        }
    }
}
