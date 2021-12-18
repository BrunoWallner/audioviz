#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    /// very important for the distributor
    /// if it is not set correctly the buffer will either grow or shrink unexpectetly
    /// which results in the draining of the buffer and thus "micro haltings"
    pub sample_rate: Option<u32>,

    /// the buffersize from the audiobackend
    /// can not be too small or to 
    pub latency: Option<u32>,

    /// list of devices can be fetched using `Capture::fetch_devices`
    pub device: String,

    /// buffer size of the distributor
    pub buffer_size: u32,

    /// the distributor will drain the buffer if its size is bigger than it
    /// acts kind of like an emergency stop to prevent the buffer to grow indefinetly
    /// results in "micro haltings"
    pub max_buffer_size: u32,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            sample_rate: None,
            latency: None,
            device: String::from("default"),
            buffer_size: 100,
            max_buffer_size: 2000,
        }
    }
}
