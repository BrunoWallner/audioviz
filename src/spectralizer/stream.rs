use crate::spectralizer::{Frequency, Processor};
use crate::spectralizer::config::{StreamConfig, ProcessorConfig};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(mpsc::Sender<Vec<Frequency>>),
    SendData(Vec<f32>),
    RequestConfig(mpsc::Sender<StreamConfig>),
    SendConfig(StreamConfig),
    RequestRefresh,
}

#[derive(Clone, Debug)]
pub struct StreamController {
    event_sender: mpsc::Sender<Event>,
}
impl StreamController {
    pub fn send_raw_data(&self, data: &[f32]) {
        self.event_sender.send(Event::SendData(data.to_vec())).unwrap();
    }

    pub fn get_frequencies(&self) -> Vec<Frequency> {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestData(tx)).unwrap();
        rx.recv().unwrap()
    }

    pub fn adjust_volume(&self, v: f32) {
        let config = self.get_config();
        let config = StreamConfig {
            processor: ProcessorConfig {
                volume: config.processor.volume * v,
                ..config.processor
            },
            ..config
        };
        self.set_config(config);
    }

    // modifying the amount of bars during runtime will result in unexpected behavior
    // unless sending 'Event::ClearBuffer' before
    // because the converter assumes that the bar amount stays the same
    // could be fixed by modifying ./src/processing/combine_buffers
    pub fn set_config(&self, config: StreamConfig) {
        self.event_sender.send(Event::SendConfig(config)).unwrap();
    }

    pub fn set_resolution(&self, number: usize) {
        let config = self.get_config();

        let wanted_conf = StreamConfig {
            processor: ProcessorConfig {
                resolution: Some(number),
                ..config.processor
            },
            ..config
        };

        self.event_sender
            .send(Event::SendConfig(wanted_conf))
            .unwrap();
    }

    pub fn get_config(&self) -> StreamConfig {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestConfig(tx)).unwrap();
        rx.recv().unwrap()
    }

    pub fn request_refresh(&self) {
        self.event_sender.send(Event::RequestRefresh).ok();
    }
}

pub struct Stream {
    event_sender: mpsc::Sender<Event>,
}
impl Stream {
    pub fn init(config: StreamConfig) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        // spawns thread that handles events
        thread::spawn(move || {
            let mut config = config;

            let cap = config.fft_resolution;
            let mut raw_buffer: Vec<f32> = Vec::with_capacity(cap);
            let mut freq_buffer: Vec<Frequency> = Vec::with_capacity(cap / 2);
            let mut gravity_time_buffer: Vec<u32> = Vec::with_capacity(cap / 2);

            loop {
                if let Ok(event) = event_receiver.recv() {
                    match event {
                        Event::RequestData(sender) => {
                            sender.send(freq_buffer.clone()).ok();
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
                                audio_data.compute_all();
                                let processed_buffer = audio_data.freq_buffer;

                                match config.gravity {
                                    Some(gravity) => {
                                        /* applies gravity to buffer */
                                        if freq_buffer.len() != processed_buffer.len() {
                                            freq_buffer = vec![Frequency::empty(); processed_buffer.len()];
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
                                            freq.volume -= gravity * 0.0025 * (gravity_time_buffer[i] as f32);
                                        }
                                    }
                                    None => {
                                        /* skips gravity */
                                        freq_buffer = processed_buffer;
                                    }
                                }
                            }
                        }
                        // end of submatch
                    }
                }
            }
        });

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

    pub fn get_controller(&self) -> StreamController {
        StreamController {
            event_sender: self.event_sender.clone()
        }
    }
}