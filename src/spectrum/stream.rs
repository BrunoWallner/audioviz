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

use crate::spectrum::config::StreamConfig;
use crate::spectrum::{processor::Processor, Frequency};

/// abstraction over `processor::Processor` with additional effects like gravity
pub struct Stream {
    pub config: StreamConfig,
    raw_buffer: Vec<f32>,
    freq_buffer: Vec<Frequency>,
    gravity_time_buffer: Vec<u32>,
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
    pub fn push_data(&mut self, mut data: Vec<f32>) {
        self.raw_buffer.append(&mut data);
    }
    pub fn get_frequencies(&mut self) -> Vec<Frequency> {
        let data = self.freq_buffer.clone();

        // additional effects get applied here, that were skiped on `self.update()`
        let mut audio_data = Processor::from_frequencies(
            self.config.clone().processor,
            data.clone(),
        );
        audio_data.bound_frequencies();
        audio_data.interpolate();
        
        audio_data.freq_buffer
    }
    /// calculates frequencies from raw data using FFT algorithm
    /// 
    /// responsible for gravity so it should be called periodicly because I have not yet implemented delta time
    pub fn update(&mut self) {
        /* Prcesses data using spectralizer::Processor */
        let fft_res: usize = self.config.fft_resolution;

        if self.raw_buffer.len() > fft_res {
            // clears unimportant buffer values that should already be processed
            // and thus reduce latency
            let diff = self.raw_buffer.len() - fft_res;
            self.raw_buffer.drain(..diff);

            let mut audio_data = Processor::from_raw_data(
                self.config.clone().processor,
                self.raw_buffer[..].to_vec(),
            );
            audio_data.apodize();
            audio_data.fft();
            audio_data.normalize_frequency_volume();

            audio_data.raw_to_freq_buffer();
            audio_data.normalize_frequency_position();
            audio_data.distribute_frequency_position();

            let processed_buffer = audio_data.freq_buffer;

            match self.config.gravity {
                Some(gravity) => {
                    /* applies gravity to buffer */
                    if self.freq_buffer.len() != processed_buffer.len() {
                        self.freq_buffer =
                            vec![Frequency::empty(); processed_buffer.len()];
                    }
                    if self.gravity_time_buffer.len() != processed_buffer.len() {
                        self.gravity_time_buffer = vec![0; processed_buffer.len()];
                    }
                    // sets value of gravity_buffer to current_buffer if current_buffer is higher
                    for i in 0..processed_buffer.len() {
                        if self.freq_buffer[i].volume < processed_buffer[i].volume {
                            self.freq_buffer[i] = processed_buffer[i].clone();
                            self.gravity_time_buffer[i] = 0;
                        } else {
                            self.gravity_time_buffer[i] += 1;
                        }
                    }

                    // apply gravity to buffer
                    for (i, freq) in self.freq_buffer.iter_mut().enumerate() {
                        let gravity: f32 = gravity * 0.0025 * (self.gravity_time_buffer[i] as f32);
                        if freq.volume - gravity >= 0.0 {
                            freq.volume -= gravity;
                        } else {
                            freq.volume = 0.0;
                            self.gravity_time_buffer[i] = 0;
                        }
                    }
                }
                None => {
                    /* skips gravity */
                    self.freq_buffer = processed_buffer;
                }
            }
        }
    }
    /*
    pub fn init(config: StreamConfig) -> Self {
        loop {
            if let Ok(event) = event_receiver.recv() {
                match event {
                    Event::RequestData(sender) => {
                        let mut audio_data = Processor::from_frequencies(
                            config.clone().processor,
                            freq_buffer.clone(),
                        );
                        audio_data.bound_frequencies();
                        audio_data.interpolate();

                        sender.send(audio_data.freq_buffer).ok();
                    }
                    Event::SendData(mut data) => {
                        raw_buffer.append(&mut data);
                    }
                    Event::RequestConfig(sender) => {
                        sender.send(config.clone()).ok();
                    }
                    Event::SendConfig(conf) => {
                        config = conf;
                    }
                    Event::RequestRefresh => {
                        /* Prcesses data using spectralizer::Processor */
                        let fft_res: usize = config.fft_resolution;

                        if raw_buffer.len() > fft_res {
                            // clears unimportant buffer values that should already be processed
                            // and thus reduce latency
                            let diff = raw_buffer.len() - fft_res;
                            raw_buffer.drain(..diff);

                            let mut audio_data = Processor::from_raw_data(
                                config.clone().processor,
                                raw_buffer[..].to_vec(),
                            );
                            audio_data.apodize();
                            audio_data.fft();
                            audio_data.normalize_frequency_volume();

                            audio_data.raw_to_freq_buffer();
                            audio_data.normalize_frequency_position();
                            audio_data.distribute_frequency_position();

                            let processed_buffer = audio_data.freq_buffer;

                            match config.gravity {
                                Some(gravity) => {
                                    /* applies gravity to buffer */
                                    if freq_buffer.len() != processed_buffer.len() {
                                        freq_buffer =
                                            vec![Frequency::empty(); processed_buffer.len()];
                                    }
                                    if gravity_time_buffer.len() != processed_buffer.len() {
                                        gravity_time_buffer = vec![0; processed_buffer.len()];
                                    }
                                    // sets value of gravity_buffer to current_buffer if current_buffer is higher
                                    for i in 0..processed_buffer.len() {
                                        if freq_buffer[i].volume < processed_buffer[i].volume {
                                            freq_buffer[i] = processed_buffer[i].clone();
                                            gravity_time_buffer[i] = 0;
                                        } else {
                                            gravity_time_buffer[i] += 1;
                                        }
                                    }

                                    // apply gravity to buffer
                                    for (i, freq) in freq_buffer.iter_mut().enumerate() {
                                        let gravity: f32 = gravity * 0.0025 * (gravity_time_buffer[i] as f32);
                                        println!("{}", gravity);
                                        if freq.volume - gravity >= 0.0 {
                                            freq.volume -= gravity;
                                        } else {
                                            freq.volume = 0.0;
                                            gravity_time_buffer[i] = 0;
                                        }
                                    }
                                }
                                None => {
                                    /* skips gravity */
                                    freq_buffer = processed_buffer;
                                }
                            }
                        }
                    } // end of submatch
                }
            }
        }

        // refresh requester
        let event_sender_clone = event_sender.clone();
        thread::spawn(move || loop {
            // receiving refresh rate from main thread
            let (tx, rx) = mpsc::channel();
            event_sender_clone.send(Event::RequestConfig(tx)).unwrap();
            let config = rx.recv().unwrap();

            thread::sleep(std::time::Duration::from_millis(
                1000 / config.refresh_rate as u64,
            ));
            event_sender_clone.send(Event::RequestRefresh).unwrap();
        });

        Stream { event_sender }
    }
    */
}
