#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// I know it can be replaced with Option<>, but I want to add things in the future
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VolumeNormalisation {
    None,
    Exponential,
    Logarithmic,

    /// both Exponential and Logarithmic
    Mixture,
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
    /// you would have to manually apply positions of frequencies and interpolation
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

    /// best looking, but might be inaccurate
    Cubic,

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
    /// neccessary so that the Audiostream knows what the hightest frequency is. (`sampling_rate` / 2)
    pub sampling_rate: u32,

    /// range of frequencies
    pub frequency_bounds: [usize; 2],

    /// number of total frequencies in processed data, None to disable up or downscaling
    /// 
    /// when `position_normalisation` and `resolution` is `None` no frequency information is lost
    /// 
    /// but when `position_normalisation` is set to anything else,
    /// information will be lost on high frequencies if no upscaling is done.
    pub resolution: Option<usize>,

    pub volume: f32,

    /// to even volume of low and high frequencies
    pub volume_normalisation: VolumeNormalisation,

    /// to mimic human hearing
    /// 
    /// might result in information loss on higher frequencies
    pub position_normalisation: PositionNormalisation,

    /// manually apply scale of frequencies
    ///
    /// frequencies around 50hz have double the scale: `vec![ (0, 1.0), (50, 2.0), (20000, 1.0) ]`
    ///
    /// this can be applied to an infinite number of frequencies: `vec![ (20, 1.0), (500, 2.0), (5000, 0.5) ... ]`
    pub manual_position_distribution: Option<Vec<(usize, f32)>>,

    /// applies positions of frequencies
    pub interpolation: Interpolation,
}
impl Default for ProcessorConfig {
    fn default() -> Self {
        ProcessorConfig {
            sampling_rate: 44_100,
            frequency_bounds: [50, 20000],
            resolution: None,
            volume: 1.0,
            volume_normalisation: VolumeNormalisation::Mixture,
            position_normalisation: PositionNormalisation::Harmonic,
            manual_position_distribution: None,
            interpolation: Interpolation::Cubic,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StreamConfig {
    pub channel_count: u16,
    pub processor: ProcessorConfig,

    /// with higher resolution comes better precision, that is mostly needed for lower frequencies
    /// at the cost of latency and 'punchiness'
    pub fft_resolution: usize,

    /// should be set to match fps of output, gravity will be affected, because I have not implemented delta-time
    pub refresh_rate: usize,

    pub gravity: Option<f32>,
}
impl Default for StreamConfig {
    fn default() -> Self {
        StreamConfig {
            channel_count: 2,
            processor: ProcessorConfig::default(),
            fft_resolution: 1024 * 2,
            refresh_rate: 60,
            gravity: Some(1.0),
        }
    }
}
