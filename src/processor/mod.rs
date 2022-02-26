pub mod filter;
use filter::{lowpass_filter, highpass_filter};

#[derive(Copy, Clone, Debug)]
pub enum Plugin {
    Lowpass{cutoff_frequency: f32},
    Highpass{cutoff_frequency: f32}
}

pub struct Processor {
    pub data: Vec<f32>,
    pub plugins: Vec<Plugin>,
    pub sampling_rate: f32,
}
impl Processor {
    pub fn process(&mut self) {
        for plugin in self.plugins.iter() {
            match plugin {
                Plugin::Lowpass{cutoff_frequency} => {
                    self.data = lowpass_filter(&self.data, self.sampling_rate, *cutoff_frequency)
                },
                Plugin::Highpass{cutoff_frequency} => {
                    self.data = highpass_filter(&self.data, self.sampling_rate, *cutoff_frequency)
                }
            }
        }
    }
}