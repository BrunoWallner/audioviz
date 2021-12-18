#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Config {
    pub sample_rate: Option<u32>,
    pub latency: Option<u32>,
    pub device: String,
    pub buffer_size: u32,
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
