pub mod processor;
pub mod config;
pub mod stream;

pub use processor::Processor;
pub use stream::{Stream, StreamController};

/// Single Frequency
/// 
/// Multiple of these are stored in a Vector,
#[derive(Clone, Debug)]
pub struct Frequency {
    pub volume: f32,

    /// Actual frequency in hz, can range from 0 to `config.sample_rate` / 2
    /// 
    /// Accuracy can vary and is not guaranteed
    pub freq: f32,

    /// Relative position of single frequency in range (0..=1)
    /// 
    /// Used to make lower freqs occupy more space than higher ones, to mimic human hearing
    /// 
    /// Should not be Important, except when distributing freqs manually
    /// 
    /// To do this manually set `config.interpolation` equal to `Interpolation::None`
    pub position: f32,
}
impl Frequency {
    pub fn empty() -> Self {
        Frequency {volume: 0.0, freq: 0.0, position: 0.0}
    }
}