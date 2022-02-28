pub mod filter;
use filter::{lowpass_filter, highpass_filter, bandpass_filter};

#[derive(Copy, Clone, Debug)]
pub struct Lowpass {
    pub cutoff_start_freq: f32,
    pub cutoff_end_freq: f32,
} impl Lowpass {
    pub fn new(cutoff_start_freq: f32, cutoff_end_freq: f32) -> Self {
        Self {
            cutoff_start_freq,
            cutoff_end_freq
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Highpass {
    pub cutoff_start_freq: f32,
    pub cutoff_end_freq: f32,
} impl Highpass {
    pub fn new(cutoff_start_freq: f32, cutoff_end_freq: f32) -> Self {
        Self {
            cutoff_start_freq,
            cutoff_end_freq
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Bandpass {
    pub low_cutoff_start_freq: f32,
    pub low_cutoff_end_freq: f32,
    pub high_cutoff_start_freq: f32,
    pub high_cutoff_end_freq: f32,
} impl Bandpass {
    pub fn new(
        low_cutoff_start_freq: f32,
        low_cutoff_end_freq: f32,
        high_cutoff_start_freq: f32,
        high_cutoff_end_freq: f32
    ) -> Self {
        Self {
            low_cutoff_start_freq,
            low_cutoff_end_freq,
            high_cutoff_start_freq,
            high_cutoff_end_freq
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Plugin {
    Lowpass(Lowpass),
    Highpass(Highpass),
    Bandpass(Bandpass)
}

pub struct Processor {
    pub data: Vec<f32>,
    pub sampling_rate: f32,
    pub plugins: Vec<Plugin>,
}
impl Processor {
    pub fn process(&mut self) {
        for plugin in self.plugins.iter() {
            match plugin {
                Plugin::Lowpass(lowpass) => {
                    self.data = lowpass_filter(&self.data, self.sampling_rate, lowpass.cutoff_start_freq, lowpass.cutoff_end_freq)
                },
                Plugin::Highpass(highpass) => {
                    self.data = highpass_filter(&self.data, self.sampling_rate, highpass.cutoff_start_freq, highpass.cutoff_end_freq)
                },
                Plugin::Bandpass(bandpass) => {
                    self.data = bandpass_filter(
                        &self.data, 
                        self.sampling_rate,
                        bandpass.low_cutoff_start_freq,
                        bandpass.low_cutoff_end_freq,
                        bandpass.high_cutoff_start_freq,
                        bandpass.high_cutoff_end_freq,
                    )
                }
            }
        }
    }
}