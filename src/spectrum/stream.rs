//! # How it works
//! ```text
//!     ┌──────────────────────────┐
//!     │ thread that consistently │
//!     │   sends data to Stream   │
//!     └──────────────────────────┘         
//!           |
//!           | data stored as `Vec<f32>`
//!           ↓
//! ┌───────────────────┐        ┌─────────────┐
//! │      Stream       │ -----> |  Processor  |
//! |                   │ <----- |             |
//! └───────────────────┘        └─────────────┘
//!        ↑ └----------┐
//!        └----------┐ |
//! get_frequencies() | | processed data stored as `Vec<Frequency>`
//!                   | ↓
//!     ┌─────────────────────────┐
//!     │thread that receives data│
//!     └─────────────────────────┘
//! ```

use super::config::StreamConfig;
use super::{processor::Processor, Frequency};
use crate::utils::seperate_channels;

/// abstraction over `processor::Processor` with additional effects like gravity
pub struct Stream {
    pub config: StreamConfig,
    raw_buffer: Vec<Vec<f32>>,
    freq_buffer: Vec<Vec<Frequency>>,
    gravity_time_buffer: Vec<Vec<u32>>,
}
impl Stream {
    pub fn new(config: StreamConfig) -> Self {
        let cap: usize = config.fft_resolution;
        Self {
            config,
            raw_buffer: Vec::with_capacity(cap),
            freq_buffer: Vec::with_capacity(cap),
            gravity_time_buffer: Vec::with_capacity(cap),
        }
    }
    pub fn push_data(&mut self, data: Vec<f32>) {
        //self.raw_buffer.append(&mut data);
        let channels: usize = self.config.channel_count as usize;
        if self.raw_buffer.len() != channels {
            self.raw_buffer = vec![vec![]; channels];
        }
        for (channel, data) in seperate_channels(&data, channels).iter().enumerate() {
            let data = &mut data.clone();
            self.raw_buffer[channel].append(data);
        }
    }
    pub fn get_frequencies(&mut self) -> Vec<Vec<Frequency>> {
        let data = self.freq_buffer.clone();

        let channels: usize = self.config.channel_count as usize;
        let mut buffer: Vec<Vec<Frequency>> = Vec::with_capacity(channels);
        // additional effects get applied here, that were skiped on `self.update()`
        for channel_data in data.iter() {
            let mut audio_data = Processor::from_frequencies(
                self.config.clone().processor,
                channel_data.clone(),
            );
            audio_data.bound_frequencies();
            audio_data.interpolate();
            
            buffer.push(audio_data.freq_buffer)
        }
        buffer
    }
    /// calculates frequencies from raw data using FFT algorithm
    /// 
    /// responsible for gravity so it should be called periodicly because I have not yet implemented delta time
    pub fn update(&mut self) {
        // processes on every channel
        let channels: usize = self.config.channel_count as usize;
        for (channel, raw_data) in self.raw_buffer.iter_mut().enumerate() {
            /* Prcesses data using spectralizer::Processor */
            let fft_res: usize = self.config.fft_resolution;

            if raw_data.len() > fft_res {
                // clears unimportant buffer values that should already be processed
                // and thus reduce latency
                let diff = raw_data.len() - fft_res;
                raw_data.drain(..diff);
    
                let mut audio_data = Processor::from_raw_data(
                    self.config.clone().processor,
                    raw_data[..].to_vec(),
                );
                audio_data.apodize();
                audio_data.fft();
                audio_data.normalize_frequency_volume();
    
                audio_data.raw_to_freq_buffer();
                audio_data.normalize_frequency_position();
                audio_data.distribute_frequency_position();
    
                let processed_buffer = audio_data.freq_buffer;
    
                // freq_buffer allocation size check
                if self.freq_buffer.len() != channels {
                    self.freq_buffer = vec![vec![Frequency::empty()]; channels];
                }
                if self.freq_buffer[channel].len() != processed_buffer.len() {
                    self.freq_buffer[channel] = vec![Frequency::empty(); processed_buffer.len()];
                }

                // gravity time allocation size check
                if self.gravity_time_buffer.len() != channels {
                    self.gravity_time_buffer = vec![vec![0]; channels];
                }
                if self.gravity_time_buffer[channel].len() != processed_buffer.len() {
                    self.gravity_time_buffer[channel] = vec![0; processed_buffer.len()];
                }

                match self.config.gravity {
                    Some(gravity) => {
                        /* applies gravity to buffer */
                        // sets value of gravity_buffer to current_buffer if current_buffer is higher
                        for i in 0..processed_buffer.len() {
                            if self.freq_buffer[channel][i].volume < processed_buffer[i].volume {
                                self.freq_buffer[channel][i] = processed_buffer[i].clone();
                                self.gravity_time_buffer[channel][i] = 0;
                            } else {
                                self.gravity_time_buffer[channel][i] += 1;
                            }
                        }
    
                        // apply gravity to buffer
                        for (i, freq) in self.freq_buffer[channel].iter_mut().enumerate() {
                            let gravity: f32 = gravity * 0.0025 * (self.gravity_time_buffer[channel][i] as f32);
                            if freq.volume - gravity >= 0.0 {
                                freq.volume -= gravity;
                            } else {
                                freq.volume = 0.0;
                                self.gravity_time_buffer[channel][i] = 0;
                            }
                        }
                    }
                    None => {
                        /* skips gravity */
                        self.freq_buffer[channel] = processed_buffer;
                    }
                }
            }
        }
    }
}
