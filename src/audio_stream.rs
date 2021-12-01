use crate::processor::*;
use crate::config::{Config, ProcessorConfig};
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone)]
pub enum Event {
    RequestData(mpsc::Sender<Vec<Frequency>>),
    SendData(Vec<f32>),
    RequestConfig(mpsc::Sender<Config>),
    SendConfig(Config),
    RequestRefresh,
}

enum ConverterEvent {
    RequestData(mpsc::Sender<Vec<Frequency>>),
    SendRawData(Vec<f32>),
    SendConfig(Config),
}

#[derive(Clone, Debug)]
pub struct AudioStreamController {
    event_sender: mpsc::Sender<Event>,
}
impl AudioStreamController {
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
        let config = Config {
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
    pub fn set_config(&self, config: Config) {
        self.event_sender.send(Event::SendConfig(config)).unwrap();
    }

    pub fn set_resolution(&self, number: usize) {
        let config = self.get_config();

        let wanted_conf = Config {
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

    pub fn get_config(&self) -> Config {
        let (tx, rx) = mpsc::channel();
        self.event_sender.send(Event::RequestConfig(tx)).unwrap();
        rx.recv().unwrap()
    }
}

pub struct AudioStream {
    event_sender: mpsc::Sender<Event>,
}
impl AudioStream {
    pub fn init(config: Config) -> Self {
        let (event_sender, event_receiver) = mpsc::channel();

        // thread that computes and converts Data
        let (converter_sender, converter_receiver) = mpsc::channel();
        let config_clone = config.clone();
        thread::spawn(move || {
            let mut buffer: Vec<f32> = Vec::new();
            let mut config: Config = config_clone;
            loop {
                match converter_receiver.recv().unwrap() {
                    ConverterEvent::RequestData(sender) => {
                        let fft_res: usize = config.fft_resolution;

                        if buffer.len() > fft_res {

                            // clears unimportant buffer values
                            let diff = buffer.len() - fft_res;
                            buffer.drain(..diff);

                            let mut audio_data = Processor::from_raw_data(
                                config.clone().processor,
                                buffer[..].to_vec(),
                            );
                            audio_data.compute_all();
                            
                            // must only iterate ONCE
                            sender.send(audio_data.freq_buffer).ok();
                        }
                    },
                    ConverterEvent::SendRawData(mut data) => {
                        buffer.append(&mut data);
                    }
                    ConverterEvent::SendConfig(conf) => {
                        config = conf;
                    }
                }
            }
        });

        // thread that receives Events
        // and sends it via a mpsc channel to requesting to thread that requested processed data
        //let event_sender_clone = event_sender.clone();
        thread::spawn(move || {
            //let mut buffer: Vec<f32> = Vec::new();
            let mut current_buffer: Vec<Frequency> = Vec::new();

            let mut gravity_buffer: Vec<Frequency> = Vec::new();
            let mut gravity_time_buffer: Vec<u32> = Vec::new();

            let mut config: Config = config;

            loop {
                match event_receiver.recv().unwrap() {
                    Event::SendData(b) => {
                        converter_sender.send(ConverterEvent::SendRawData(b)).unwrap();
                    }
                    Event::RequestData(sender) => {
                        let buf = match config.gravity {
                            Some(_) => {
                                gravity_buffer.clone()
                            }
                            None => {
                                current_buffer.clone()
                            }
                        };

                        let mut processor = Processor::from_frequencies(config.processor.clone(), buf);
                        processor.interpolate();
                        processor.bound_frequencies();
                        processor.apply_resolution();
                        let buf = processor.freq_buffer;

                        // finally sends buffer to request channel
                        sender.send(buf).unwrap();
                    },
                    Event::RequestRefresh => {
                        // request data from converter thread
                        let (tx, rx) = mpsc::channel();
                        converter_sender.send(ConverterEvent::RequestData(tx)).unwrap();
                        current_buffer = match rx.recv() {
                            Ok(buf) => buf,
                            Err(_) => current_buffer
                        };

                        /* Gravity */
                        match config.gravity {
                            Some(gravity) => {
                                if gravity_buffer.len() != current_buffer.len() {
                                    gravity_buffer = vec![Frequency::empty(); current_buffer.len()];
                                }
                                if gravity_time_buffer.len() != current_buffer.len() {
                                    gravity_time_buffer = vec![0; current_buffer.len()];
                                }

                                // sets value of gravity_buffer to current_buffer if current_buffer is higher
                                for i in 0..current_buffer.len() {
                                    if gravity_buffer[i].volume < current_buffer[i].volume {
                                        gravity_buffer[i] = current_buffer[i].clone();
                                        gravity_time_buffer[i] = 0;
                                    } else {
                                        gravity_time_buffer[i] += 1;
                                    }
                                }

                                // apply gravity to buffer
                                for (i, freq) in gravity_buffer.iter_mut().enumerate() {
                                    freq.volume -= gravity * 0.0025 * (gravity_time_buffer[i] as f32);
                                }
                            }
                            None => (),
                        }
                    }
                    Event::RequestConfig(sender) => {
                        sender.send(config.clone()).unwrap();
                    }
                    Event::SendConfig(c) => {
                        config = c.clone();
                        converter_sender.send(ConverterEvent::SendConfig(c)).unwrap();
                    }
                }
            }
        });
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

        AudioStream { event_sender }
    }
    pub fn get_controller(&self) -> AudioStreamController {
        AudioStreamController {
            event_sender: self.event_sender.clone()
        }
    }
}

